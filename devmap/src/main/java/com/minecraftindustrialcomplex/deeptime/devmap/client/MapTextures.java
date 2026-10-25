// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.BaseImage;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapState;
import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.mojang.blaze3d.platform.NativeImage;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.resources.ResourceLocation;

import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The dev map's two textures, made on the client's main thread: the picture shown (satellite or
 * elevation, {@link MapContext#image}) as it came from the server (block space), and the globe,
 * drawn on the CPU by Deep Time's own {@link GlobeProjection#render} from a latitude/longitude
 * picture of it (built once per server picture on a background thread and kept per layer, so
 * switching back and forth does not rebuild). Without a picture the globe is drawn from a plain one.
 */
final class MapTextures {

    static final ResourceLocation MAP = ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "dynamic/map");
    static final ResourceLocation GLOBE = ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "dynamic/globe");

    /** The latitude/longitude picture's width (its height follows the keyframes' π : 1). */
    static final int PICTURE_W = 1024;

    private static DynamicTexture map;
    private static DevMapState.Image mapOf;

    /** A latitude/longitude picture and the server picture it was built from. */
    private record LatLon(DevMapState.Image of, int[] abgr) {}

    private static final Map<MapLayer, LatLon> PICTURES = new ConcurrentHashMap<>();
    private static final Set<DevMapState.Image> BUILDING = ConcurrentHashMap.newKeySet();
    private static final int[] PLAIN = plain();

    private static DynamicTexture globe;
    private static int globeSize;
    private static double globeLat = Double.NaN, globeLon = Double.NaN;
    private static Object globeSource;

    private MapTextures() {}

    private static int[] plain() {
        int[] p = new int[8 * 3];
        java.util.Arrays.fill(p, 0xff6a4a2a); // ABGR: a slate blue
        return p;
    }

    /** Uploads {@code im} as the map texture when it is not already; false when there is no image. */
    static boolean map(DevMapState.Image im) {
        if (im == null)
            return false;
        if (mapOf == im && map != null)
            return true;
        int w = im.frame().w(), h = im.frame().h();
        if (map == null || map.getPixels() == null || map.getPixels().getWidth() != w || map.getPixels().getHeight() != h) {
            if (map != null)
                Minecraft.getInstance().getTextureManager().release(MAP);
            map = new DynamicTexture(w, h, false);
            Minecraft.getInstance().getTextureManager().register(MAP, map);
        }
        NativeImage img = map.getPixels();
        int[] px = im.abgr();
        for (int y = 0; y < h; y++)
            for (int x = 0; x < w; x++)
                img.setPixelRGBA(x, y, px[y * w + x]);
        map.upload();
        mapOf = im;
        return true;
    }

    /**
     * The latitude/longitude picture of {@code im} (width {@link #PICTURE_W}), or null while it is
     * being built (the first call starts the build).
     */
    static int[] picture(DevMapState.Image im, WorldProjection p) {
        if (im == null)
            return null;
        LatLon built = PICTURES.get(im.layer());
        if (built != null && built.of() == im)
            return built.abgr();
        if (BUILDING.add(im)) {
            int w = PICTURE_W, h = MapPicture.height(w);
            Thread t = new Thread(() -> {
                try {
                    long t0 = System.nanoTime();
                    int[] pic = BaseImage.latLonPicture(im.abgr(), im.frame(), p, w, h);
                    PICTURES.put(im.layer(), new LatLon(im, pic));
                    DevMap.LOGGER.info("[devmap] {} globe picture {}x{} built in {} ms", im.layer().id(), w, h,
                            (System.nanoTime() - t0) / 1_000_000);
                } finally {
                    BUILDING.remove(im);
                }
            }, "deeptime_devmap-picture");
            t.setDaemon(true);
            t.start();
        }
        return null;
    }

    /**
     * Makes the globe texture show the planet centred on (lat, lon) at {@code size} pixels from
     * {@code pic} ({@code w × h}, or the plain picture when null); re-renders only when something
     * changed.
     */
    static void globe(int[] pic, double lat, double lon, int size) {
        Object source = pic == null ? PLAIN : pic;
        int w = pic == null ? 8 : PICTURE_W;
        int h = pic == null ? 3 : MapPicture.height(PICTURE_W);
        if (globe != null && globeSize == size && globeLat == lat && globeLon == lon && globeSource == source)
            return;
        if (globe == null || globeSize != size) {
            if (globe != null)
                Minecraft.getInstance().getTextureManager().release(GLOBE);
            globe = new DynamicTexture(size, size, false);
            Minecraft.getInstance().getTextureManager().register(GLOBE, globe);
            globeSize = size;
        }
        int[] dst = new int[size * size];
        new GlobeProjection(lat, lon).render((int[]) source, w, h, dst, size);
        NativeImage img = globe.getPixels();
        for (int y = 0; y < size; y++)
            for (int x = 0; x < size; x++)
                img.setPixelRGBA(x, y, dst[y * size + x]);
        globe.upload();
        globeLat = lat;
        globeLon = lon;
        globeSource = source;
    }

    /** Forgets everything (the client left the server). */
    static void reset() {
        mapOf = null;
        PICTURES.clear();
        globeSource = null;
    }
}
