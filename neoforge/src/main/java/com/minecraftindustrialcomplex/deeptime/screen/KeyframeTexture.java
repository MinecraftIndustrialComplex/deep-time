// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.blaze3d.platform.NativeImage;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.resources.ResourceLocation;

import java.util.concurrent.atomic.AtomicInteger;

/**
 * A GUI texture filled from ABGR pixels (plan §7.12: keyframe → {@code NativeImage} →
 * {@code DynamicTexture.upload()} → {@code GuiGraphics.blit}). Keyframes arrive as RGBA8 bytes;
 * read as little-endian ints they are already {@code NativeImage}'s ABGR order. Render thread only;
 * {@link #close()} releases the GPU texture.
 */
final class KeyframeTexture implements AutoCloseable {

    private static final AtomicInteger NEXT = new AtomicInteger();

    private final ResourceLocation id;
    private DynamicTexture texture;
    private int w, h;

    KeyframeTexture(String name) {
        this.id = ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "dynamic/" + name + "_" + NEXT.incrementAndGet());
    }

    /** Uploads {@code abgr} ({@code w × h}); reallocates when the size changes. */
    void upload(int[] abgr, int w, int h) {
        if (texture == null || this.w != w || this.h != h) {
            close();
            texture = new DynamicTexture(w, h, false);
            this.w = w;
            this.h = h;
            Minecraft.getInstance().getTextureManager().register(id, texture);
        }
        NativeImage img = texture.getPixels();
        if (img == null)
            return;
        for (int y = 0; y < h; y++)
            for (int x = 0; x < w; x++)
                img.setPixelRGBA(x, y, abgr[y * w + x]);
        texture.upload();
    }

    boolean ready() {
        return texture != null;
    }

    ResourceLocation id() {
        return id;
    }

    int width() {
        return w;
    }

    int height() {
        return h;
    }

    @Override
    public void close() {
        if (texture != null) {
            Minecraft.getInstance().getTextureManager().release(id);
            texture = null;
        }
    }
}
