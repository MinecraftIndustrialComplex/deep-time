// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deep Time's GPU compute path (`docs/design/gpu-compute.md`): one wgpu device per process,
//! picked automatically (a discrete GPU, then an integrated one; a software adapter such as
//! Mesa's lavapipe only when asked for), and what every ported kernel needs around it: storage
//! buffers allocated under an out-of-memory scope, uploads and staged downloads, compute
//! pipelines with explicit layouts, and dispatches split into bands of rows so that no dispatch
//! runs long enough for an operating system's GPU watchdog (Windows' TDR, about 2 s) to reset the
//! device.
//!
//! Nothing here panics on a driver's behalf: a validation error, a failed allocation, a hung or a
//! lost device comes back as `Err(String)`, and the caller runs the work on the CPU instead. The
//! crate knows nothing of Minecraft, JNI or the simulation; the kernels live next to the CPU code
//! they mirror (`deeptime-core`, feature `gpu`).

use std::future::Future;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

pub use bytemuck;
pub use wgpu;

/// The kernels' workgroups are `WORKGROUP × WORKGROUP` invocations (256, the most WebGPU
/// guarantees).
pub const WORKGROUP: u32 = 16;
/// The most cells one dispatch covers. A stencil over a band of 4 M cells takes a few
/// milliseconds on a mid-range GPU and well under a second on a weak integrated one, far inside
/// the 2 s watchdog.
pub const BAND_CELLS: usize = 1 << 22;
/// A wait for the device longer than this counts as a hung device.
pub const WAIT_LIMIT: Duration = Duration::from_secs(120);
/// Environment: a case-insensitive part of the adapter's name to use (e.g. `8060S`, `llvmpipe`).
/// Naming a software adapter allows it even when the mode is `auto`.
pub const ENV_ADAPTER: &str = "DEEPTIME_GPU_ADAPTER";
/// Environment: MiB of device memory the process may keep resident (default 3072).
pub const ENV_MEM_MIB: &str = "DEEPTIME_GPU_MEM_MIB";
const DEFAULT_BUDGET: u64 = 3 << 30;
/// Downloads go through staging buffers of at most this size.
const STAGING_BYTES: u64 = 64 << 20;

/// An open device and what the kernels need to know about it.
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: wgpu::AdapterInfo,
    pub limits: wgpu::Limits,
    /// The adapter's memory is the host's (an integrated or a software adapter): what it holds
    /// counts against the session's memory cap like any host array.
    pub shared_memory: bool,
    /// Bytes the process may keep resident on the device ([`ENV_MEM_MIB`]).
    pub budget: u64,
    band_layout: wgpu::BindGroupLayout,
    fault: Arc<Mutex<Option<String>>>,
    lost: Arc<AtomicBool>,
}

impl std::fmt::Debug for Gpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gpu")
            .field("adapter", &self.describe())
            .field("budget", &self.budget)
            .finish_non_exhaustive()
    }
}

type Slot = Option<Result<Arc<Gpu>, String>>;
/// The process's devices: `[hardware only, software allowed]`. A failure is cached too (the
/// adapters do not change while the game runs); a lost device is opened again on the next ask.
static DEVICES: Mutex<[Slot; 2]> = Mutex::new([None, None]);

/// The process's device, opened on first use. `allow_software` also accepts a CPU adapter
/// (lavapipe, WARP), which is slower than the CPU path and so only right for tests and for a
/// forced `gpu` mode.
pub fn shared(allow_software: bool) -> Result<Arc<Gpu>, String> {
    let mut slots = DEVICES.lock().unwrap_or_else(|p| p.into_inner());
    let slot = &mut slots[usize::from(allow_software)];
    if matches!(slot, Some(Ok(g)) if g.is_lost()) {
        *slot = None;
    }
    slot.get_or_insert_with(|| open(allow_software).map(Arc::new))
        .clone()
}

fn open(allow_software: bool) -> Result<Gpu, String> {
    std::panic::catch_unwind(|| open_inner(allow_software)).unwrap_or_else(|p| {
        let why = p
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        Err(format!("the GPU driver panicked while opening: {why}"))
    })
}

/// Smaller is better: the device type first, then the platform's native backend.
fn rank(info: &wgpu::AdapterInfo) -> u32 {
    use wgpu::DeviceType as T;
    let ty = match info.device_type {
        T::DiscreteGpu => 0,
        T::IntegratedGpu => 1,
        T::VirtualGpu => 2,
        T::Other => 3,
        T::Cpu => 4,
    };
    let native = if cfg!(windows) {
        wgpu::Backend::Dx12
    } else if cfg!(target_vendor = "apple") {
        wgpu::Backend::Metal
    } else {
        wgpu::Backend::Vulkan
    };
    2 * ty + u32::from(info.backend != native)
}

fn open_inner(allow_software: bool) -> Result<Gpu, String> {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12;
    desc.flags = wgpu::InstanceFlags::empty();
    // Allocations past 90 % of what the driver reports free fail cleanly (the CPU takes over)
    // instead of evicting the game's textures.
    desc.memory_budget_thresholds = wgpu::MemoryBudgetThresholds {
        for_resource_creation: Some(90),
        for_device_loss: None,
    };
    let instance = wgpu::Instance::new(desc.with_env());
    let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let want = std::env::var(ENV_ADAPTER)
        .ok()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    let mut seen = Vec::new();
    let mut best: Option<(u32, wgpu::Adapter)> = None;
    for a in adapters {
        let info = a.get_info();
        seen.push(format!(
            "{} ({:?}, {:?})",
            info.name, info.backend, info.device_type
        ));
        if let Some(w) = &want {
            if !info.name.to_lowercase().contains(w.as_str()) {
                continue;
            }
        } else if info.device_type == wgpu::DeviceType::Cpu && !allow_software {
            continue;
        }
        let caps = a.get_downlevel_capabilities();
        if !caps.flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS) {
            continue;
        }
        let score = rank(&info);
        if best.as_ref().is_none_or(|(s, _)| score < *s) {
            best = Some((score, a));
        }
    }
    let Some((_, adapter)) = best else {
        return Err(if seen.is_empty() {
            "no GPU adapter (no Vulkan, Metal or DX12 driver found)".into()
        } else {
            format!("no usable GPU adapter among {}", seen.join("; "))
        });
    };
    let info = adapter.get_info();
    // Everything the adapter offers: the kernels need large storage buffers, and asking for the
    // adapter's own limits always succeeds.
    let limits = adapter.limits();
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("deeptime"),
        required_limits: limits.clone(),
        memory_hints: wgpu::MemoryHints::Performance,
        ..Default::default()
    }))
    .map_err(|e| format!("{}: {e}", info.name))?;
    let fault = Arc::new(Mutex::new(None::<String>));
    let lost = Arc::new(AtomicBool::new(false));
    {
        // wgpu's default handler panics; this one keeps the first error for the next check.
        let fault = fault.clone();
        device.on_uncaptured_error(Arc::new(move |e: wgpu::Error| {
            let mut f = fault.lock().unwrap_or_else(|p| p.into_inner());
            f.get_or_insert_with(|| e.to_string());
        }));
    }
    {
        let (fault, lost) = (fault.clone(), lost.clone());
        device.set_device_lost_callback(move |reason, msg| {
            lost.store(true, Ordering::SeqCst);
            let mut f = fault.lock().unwrap_or_else(|p| p.into_inner());
            f.get_or_insert_with(|| format!("device lost ({reason:?}): {msg}"));
        });
    }
    let budget = std::env::var(ENV_MEM_MIB)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map_or(DEFAULT_BUDGET, |m| m << 20);
    let shared_memory = matches!(
        info.device_type,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::Cpu
    );
    let band_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("deeptime band"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: NonZeroU64::new(16),
            },
            count: None,
        }],
    });
    Ok(Gpu {
        device,
        queue,
        info,
        limits,
        shared_memory,
        budget,
        band_layout,
        fault,
        lost,
    })
}

/// Runs a future that completes without an executor (wgpu's native futures resolve on the spot).
pub fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

/// How a kernel uses one of its buffers (bindings 0, 1, … of group 0, in order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    Uniform,
    Read,
    /// `read_write` storage.
    Write,
}

/// A compute pipeline and its group-0 layout. Group 1 is always the band ([`Bands`]): WGSL
/// `@group(1) @binding(0) var<uniform> band: Band;` with `struct Band { row0: u32, row1: u32,
/// pad0: u32, pad1: u32 }`, the rows `[row0, row1)` of this dispatch.
pub struct Kernel {
    pub name: &'static str,
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

/// The rows of a `w × h` grid in dispatches of at most [`BAND_CELLS`] cells.
pub struct Bands {
    group: wgpu::BindGroup,
    stride: u32,
    rows: Vec<(u32, u32)>,
    w: u32,
}

impl Bands {
    /// Records `k` over the whole grid, one dispatch per band, into `pass`.
    pub fn dispatch(&self, pass: &mut wgpu::ComputePass<'_>, k: &Kernel, group: &wgpu::BindGroup) {
        pass.set_pipeline(&k.pipeline);
        pass.set_bind_group(0, group, &[]);
        for (b, &(r0, r1)) in self.rows.iter().enumerate() {
            pass.set_bind_group(1, &self.group, &[b as u32 * self.stride]);
            pass.dispatch_workgroups(self.w.div_ceil(WORKGROUP), (r1 - r0).div_ceil(WORKGROUP), 1);
        }
    }

    pub fn count(&self) -> usize {
        self.rows.len()
    }
}

/// A buffer the host reads results from, kept for repeated read-backs: [`Gpu::download`]
/// allocates its staging buffer per call, which a solver reading back every iteration should not.
pub struct Readback {
    buf: wgpu::Buffer,
}

impl Readback {
    pub fn size(&self) -> u64 {
        self.buf.size()
    }
}

impl Gpu {
    /// Adapter, backend, type and driver, for logs and run notes.
    pub fn describe(&self) -> String {
        let i = &self.info;
        let driver = [i.driver.as_str(), i.driver_info.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{} ({:?}, {:?}, {driver})",
            i.name, i.backend, i.device_type
        )
    }

    pub fn is_lost(&self) -> bool {
        self.lost.load(Ordering::SeqCst)
    }

    /// The first error the device reported since the last check, or that it is lost.
    pub fn check(&self) -> Result<(), String> {
        let fault = self.fault.lock().unwrap_or_else(|p| p.into_inner()).take();
        if let Some(e) = fault {
            return Err(e);
        }
        if self.is_lost() {
            return Err("the GPU device was lost".into());
        }
        Ok(())
    }

    /// Waits until `submission` (None: everything submitted) has run, then [`Gpu::check`]s.
    pub fn wait(&self, submission: Option<wgpu::SubmissionIndex>) -> Result<(), String> {
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: submission,
                timeout: Some(WAIT_LIMIT),
            })
            .map_err(|e| format!("waiting for the GPU: {e}"))?;
        self.check()
    }

    /// Whether one buffer of `bytes` is within the adapter's limits.
    pub fn buffer_fits(&self, bytes: u64) -> bool {
        bytes <= self.limits.max_storage_buffer_binding_size && bytes <= self.limits.max_buffer_size
    }

    /// A storage buffer of `bytes` (rounded up to 4), or why there is none.
    pub fn storage(&self, label: &str, bytes: u64) -> Result<wgpu::Buffer, String> {
        let size = bytes.max(4).next_multiple_of(4);
        if !self.buffer_fits(size) {
            return Err(format!(
                "{label}: {size} bytes exceed the adapter's largest storage buffer ({} bytes)",
                self.limits.max_storage_buffer_binding_size
            ));
        }
        let oom = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let invalid = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let e1 = block_on(invalid.pop());
        let e2 = block_on(oom.pop());
        match e1.or(e2) {
            Some(e) => Err(format!("{label}: {e}")),
            None => Ok(b),
        }
    }

    /// A uniform buffer holding `value`.
    pub fn uniform<T: bytemuck::Pod>(&self, label: &str, value: &T) -> wgpu::Buffer {
        let bytes = bytemuck::bytes_of(value);
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (bytes.len() as u64).next_multiple_of(16),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&b, 0, bytes);
        b
    }

    /// Queues `data` into the start of `dst` (it lands before the next submission runs).
    pub fn upload<T: bytemuck::Pod>(&self, dst: &wgpu::Buffer, data: &[T]) {
        if !data.is_empty() {
            self.queue.write_buffer(dst, 0, bytemuck::cast_slice(data));
        }
    }

    /// Copies the start of `src` into `out`, through staging buffers of at most 64 MiB; waits.
    pub fn download<T: bytemuck::Pod>(
        &self,
        src: &wgpu::Buffer,
        out: &mut [T],
    ) -> Result<(), String> {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(out);
        let total = bytes.len() as u64;
        if total == 0 {
            return Ok(());
        }
        if !total.is_multiple_of(4) {
            return Err(format!("download of {total} bytes: not a multiple of 4"));
        }
        let chunk = total.min(STAGING_BYTES);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("deeptime staging"),
            size: chunk,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut off = 0u64;
        while off < total {
            let n = (total - off).min(chunk);
            let mut enc = self.device.create_command_encoder(&Default::default());
            enc.copy_buffer_to_buffer(src, off, &staging, 0, n);
            let idx = self.queue.submit([enc.finish()]);
            let (tx, rx) = mpsc::channel();
            staging.map_async(wgpu::MapMode::Read, 0..n, move |r| {
                let _ = tx.send(r);
            });
            self.wait(Some(idx))?;
            let mapped = match rx.try_recv() {
                Ok(r) => r,
                Err(_) => {
                    self.wait(None)?;
                    rx.try_recv()
                        .map_err(|_| "download: the buffer was never mapped".to_string())?
                }
            };
            mapped.map_err(|e| format!("download: {e}"))?;
            {
                let view = staging
                    .get_mapped_range(0..n)
                    .map_err(|e| format!("download: {e}"))?;
                bytes[off as usize..(off + n) as usize].copy_from_slice(&view);
            }
            staging.unmap();
            off += n;
        }
        Ok(())
    }

    /// A read-back buffer of `bytes` (rounded up to 4), or why there is none.
    pub fn readback(&self, label: &str, bytes: u64) -> Result<Readback, String> {
        let size = bytes.max(4).next_multiple_of(4);
        if size > self.limits.max_buffer_size {
            return Err(format!(
                "{label}: {size} bytes exceed the adapter's largest buffer ({} bytes)",
                self.limits.max_buffer_size
            ));
        }
        let oom = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let invalid = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let e1 = block_on(invalid.pop());
        let e2 = block_on(oom.pop());
        match e1.or(e2) {
            Some(e) => Err(format!("{label}: {e}")),
            None => Ok(Readback { buf }),
        }
    }

    /// Ends `enc` with copies of the start of each source (`(buffer, bytes)`, placed one after
    /// another in `rb`), submits it, waits, and hands `read` the copied bytes (one submission and
    /// one wait for the work and its results).
    pub fn submit_and_read<R>(
        &self,
        mut enc: wgpu::CommandEncoder,
        rb: &Readback,
        copies: &[(&wgpu::Buffer, u64)],
        read: impl FnOnce(&[u8]) -> R,
    ) -> Result<R, String> {
        let total: u64 = copies.iter().map(|c| c.1).sum();
        if total > rb.buf.size() || copies.iter().any(|c| !c.1.is_multiple_of(4)) {
            return Err(format!(
                "read-back of {total} bytes into {} (each a multiple of 4)",
                rb.buf.size()
            ));
        }
        let mut off = 0u64;
        for &(src, n) in copies {
            if n > 0 {
                enc.copy_buffer_to_buffer(src, 0, &rb.buf, off, n);
            }
            off += n;
        }
        let idx = self.queue.submit([enc.finish()]);
        if total == 0 {
            self.wait(Some(idx))?;
            return Ok(read(&[]));
        }
        let (tx, rx) = mpsc::channel();
        rb.buf.map_async(wgpu::MapMode::Read, 0..total, move |r| {
            let _ = tx.send(r);
        });
        self.wait(Some(idx))?;
        let mapped = match rx.try_recv() {
            Ok(r) => r,
            Err(_) => {
                self.wait(None)?;
                rx.try_recv()
                    .map_err(|_| "read-back: the buffer was never mapped".to_string())?
            }
        };
        mapped.map_err(|e| format!("read-back: {e}"))?;
        let out = {
            let view = rb
                .buf
                .get_mapped_range(0..total)
                .map_err(|e| format!("read-back: {e}"))?;
            read(&view)
        };
        rb.buf.unmap();
        Ok(out)
    }

    /// Compiles WGSL `source`.
    pub fn module(&self, label: &str, source: &str) -> Result<wgpu::ShaderModule, String> {
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let m = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        match block_on(scope.pop()) {
            Some(e) => Err(format!("{label}: {e}")),
            None => Ok(m),
        }
    }

    /// The pipeline of `entry` in `module`, whose group 0 holds one buffer per `uses` entry.
    pub fn kernel(
        &self,
        module: &wgpu::ShaderModule,
        entry: &'static str,
        uses: &[Use],
    ) -> Result<Kernel, String> {
        let entries: Vec<_> = uses
            .iter()
            .enumerate()
            .map(|(b, u)| wgpu::BindGroupLayoutEntry {
                binding: b as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: match u {
                        Use::Uniform => wgpu::BufferBindingType::Uniform,
                        Use::Read => wgpu::BufferBindingType::Storage { read_only: true },
                        Use::Write => wgpu::BufferBindingType::Storage { read_only: false },
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(entry),
                entries: &entries,
            });
        let pl = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(entry),
                bind_group_layouts: &[Some(&layout), Some(&self.band_layout)],
                immediate_size: 0,
            });
        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pl),
                module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            });
        match block_on(scope.pop()) {
            Some(e) => Err(format!("{entry}: {e}")),
            None => Ok(Kernel {
                name: entry,
                pipeline,
                layout,
            }),
        }
    }

    /// The group-0 bind group of `k` over `buffers` (in binding order).
    pub fn bind(&self, k: &Kernel, buffers: &[&wgpu::Buffer]) -> wgpu::BindGroup {
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(b, buf)| wgpu::BindGroupEntry {
                binding: b as u32,
                resource: buf.as_entire_binding(),
            })
            .collect();
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(k.name),
            layout: &k.layout,
            entries: &entries,
        })
    }

    /// The bands of a `w × h` grid: rows in multiples of [`WORKGROUP`], at most [`BAND_CELLS`]
    /// cells each.
    pub fn bands(&self, w: usize, h: usize) -> Bands {
        let wg = WORKGROUP as usize;
        let per = (BAND_CELLS / w.max(1) / wg * wg).max(wg);
        let rows: Vec<(u32, u32)> = (0..h)
            .step_by(per)
            .map(|r0| (r0 as u32, (r0 + per).min(h) as u32))
            .collect();
        let stride = self.limits.min_uniform_buffer_offset_alignment.max(16);
        let mut data = vec![0u8; rows.len().max(1) * stride as usize];
        for (b, &(r0, r1)) in rows.iter().enumerate() {
            let o = b * stride as usize;
            data[o..o + 4].copy_from_slice(&r0.to_le_bytes());
            data[o + 4..o + 8].copy_from_slice(&r1.to_le_bytes());
        }
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("deeptime bands"),
            size: data.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, &data);
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deeptime bands"),
            layout: &self.band_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZeroU64::new(16),
                }),
            }],
        });
        Bands {
            group,
            stride,
            rows,
            w: w as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOUBLE: &str = r#"
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
struct P { w: u32, h: u32, pad0: u32, pad1: u32 }
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var<storage, read> src: array<f32>;
@group(0) @binding(2) var<storage, read_write> dst: array<f32>;
@group(1) @binding(0) var<uniform> band: Band;
@compute @workgroup_size(16, 16)
fn double(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    if (id.x >= p.w || j >= band.row1) { return; }
    let c = j * p.w + id.x;
    dst[c] = 2.0 * src[c] + f32(j);
}
"#;

    /// The plumbing end to end on whatever adapter there is (lavapipe on the test VM); skipped
    /// without one.
    #[test]
    fn a_kernel_runs_in_bands_and_comes_back() {
        let gpu = match shared(true) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("skipped: {e}");
                return;
            }
        };
        eprintln!("adapter: {}", gpu.describe());
        // Wide enough that the grid takes two bands.
        let (w, h) = (1usize << 17, 40usize);
        let bands = gpu.bands(w, h);
        assert_eq!(bands.count(), 2);
        let src: Vec<f32> = (0..w * h).map(|c| (c % 1000) as f32).collect();
        let a = gpu.storage("src", (w * h * 4) as u64).unwrap();
        let b = gpu.storage("dst", (w * h * 4) as u64).unwrap();
        gpu.upload(&a, &src);
        let params = gpu.uniform("p", &[w as u32, h as u32, 0, 0]);
        let m = gpu.module("double", DOUBLE).unwrap();
        let k = gpu
            .kernel(&m, "double", &[Use::Uniform, Use::Read, Use::Write])
            .unwrap();
        let g = gpu.bind(&k, &[&params, &a, &b]);
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            bands.dispatch(&mut pass, &k, &g);
        }
        gpu.queue.submit([enc.finish()]);
        let mut out = vec![0.0f32; w * h];
        gpu.download(&b, &mut out).unwrap();
        for (c, (&o, &s)) in out.iter().zip(&src).enumerate() {
            assert_eq!(o, 2.0 * s + (c / w) as f32, "cell {c}");
        }
    }

    #[test]
    fn a_bad_shader_is_an_error_not_a_panic() {
        let Ok(gpu) = shared(true) else {
            return;
        };
        assert!(gpu.module("bad", "fn nope( {").is_err());
        assert!(gpu.check().is_ok());
    }
}
