// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.blaze3d.vertex.Tesselator;
import com.mojang.blaze3d.vertex.VertexConsumer;
import com.mojang.blaze3d.vertex.VertexFormat;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.renderer.GameRenderer;
import net.minecraft.client.renderer.RenderType;
import net.minecraft.resources.ResourceLocation;
import org.joml.Matrix4f;

/**
 * Drawing helpers in GUI coordinates with fractional positions (GuiGraphics' own take whole
 * pixels): coloured quads, lines, triangles and markers into the GUI buffer (drawn at the next
 * {@code flush}), and a textured quad drawn at once. The GUI render type culls back faces, so every
 * quad is emitted in the winding {@code GuiGraphics.fill} uses.
 */
final class Draw {

    private Draw() {}

    /** One physical pixel in GUI units. */
    static float px() {
        return (float) (1.0 / Minecraft.getInstance().getWindow().getGuiScale());
    }

    static void quad(GuiGraphics g, double x1, double y1, double x2, double y2, double x3, double y3, double x4, double y4, int argb) {
        VertexConsumer vc = g.bufferSource().getBuffer(RenderType.gui());
        Matrix4f m = g.pose().last().pose();
        // Shoelace in screen space (y down): GuiGraphics.fill's winding has a negative sum.
        double area = (x1 * y2 - x2 * y1) + (x2 * y3 - x3 * y2) + (x3 * y4 - x4 * y3) + (x4 * y1 - x1 * y4);
        if (area <= 0) {
            vc.addVertex(m, (float) x1, (float) y1, 0).setColor(argb);
            vc.addVertex(m, (float) x2, (float) y2, 0).setColor(argb);
            vc.addVertex(m, (float) x3, (float) y3, 0).setColor(argb);
            vc.addVertex(m, (float) x4, (float) y4, 0).setColor(argb);
        } else {
            vc.addVertex(m, (float) x4, (float) y4, 0).setColor(argb);
            vc.addVertex(m, (float) x3, (float) y3, 0).setColor(argb);
            vc.addVertex(m, (float) x2, (float) y2, 0).setColor(argb);
            vc.addVertex(m, (float) x1, (float) y1, 0).setColor(argb);
        }
    }

    static void rect(GuiGraphics g, double x1, double y1, double x2, double y2, int argb) {
        quad(g, x1, y1, x1, y2, x2, y2, x2, y1, argb);
    }

    static void line(GuiGraphics g, double x1, double y1, double x2, double y2, double width, int argb) {
        double dx = x2 - x1, dy = y2 - y1, len = Math.hypot(dx, dy);
        if (len < 1e-9)
            return;
        double nx = -dy / len * width / 2, ny = dx / len * width / 2;
        quad(g, x1 - nx, y1 - ny, x1 + nx, y1 + ny, x2 + nx, y2 + ny, x2 - nx, y2 - ny, argb);
    }

    /** A dashed line: dashes of {@code dash} GUI units with gaps as long. */
    static void dashed(GuiGraphics g, double x1, double y1, double x2, double y2, double width, double dash, int argb) {
        double len = Math.hypot(x2 - x1, y2 - y1);
        if (len < 1e-9)
            return;
        int n = (int) Math.ceil(len / (2 * dash));
        if (n > 4000) {
            line(g, x1, y1, x2, y2, width, argb);
            return;
        }
        for (int i = 0; i < n; i++) {
            double a = 2 * dash * i / len, b = Math.min(1.0, (2 * dash * i + dash) / len);
            line(g, x1 + (x2 - x1) * a, y1 + (y2 - y1) * a, x1 + (x2 - x1) * b, y1 + (y2 - y1) * b, width, argb);
        }
    }

    static void triangle(GuiGraphics g, double x1, double y1, double x2, double y2, double x3, double y3, int argb) {
        quad(g, x1, y1, x2, y2, x3, y3, x3, y3, argb);
    }

    /** An arrow head at (x, y) pointing along the screen direction (dx, dy), {@code size} GUI units long. */
    static void arrow(GuiGraphics g, double x, double y, double dx, double dy, double size, int fill, int outline) {
        double len = Math.hypot(dx, dy);
        if (len < 1e-9) {
            dx = 0;
            dy = -1;
            len = 1;
        }
        dx /= len;
        dy /= len;
        double px = -dy, py = dx;
        double tx = x + dx * size, ty = y + dy * size;
        double bx = x - dx * size * 0.5, by = y - dy * size * 0.5;
        double o = size * 0.25;
        triangle(g, tx + dx * o, ty + dy * o, bx + px * (size * 0.6 + o) - dx * o, by + py * (size * 0.6 + o) - dy * o,
                bx - px * (size * 0.6 + o) - dx * o, by - py * (size * 0.6 + o) - dy * o, outline);
        triangle(g, tx, ty, bx + px * size * 0.6, by + py * size * 0.6, bx - px * size * 0.6, by - py * size * 0.6, fill);
    }

    /** A filled diamond marker. */
    static void diamond(GuiGraphics g, double x, double y, double r, int argb) {
        quad(g, x, y - r, x - r, y, x, y + r, x + r, y, argb);
    }

    /** A ring marker (an octagon outline). */
    static void ring(GuiGraphics g, double x, double y, double r, double width, int argb) {
        int n = 16;
        for (int i = 0; i < n; i++) {
            double a = 2 * Math.PI * i / n, b = 2 * Math.PI * (i + 1) / n;
            line(g, x + r * Math.cos(a), y + r * Math.sin(a), x + r * Math.cos(b), y + r * Math.sin(b), width, argb);
        }
    }

    /** A cross marker. */
    static void cross(GuiGraphics g, double x, double y, double r, double width, int argb) {
        line(g, x - r, y - r, x + r, y + r, width, argb);
        line(g, x - r, y + r, x + r, y - r, width, argb);
    }

    /**
     * Draws texture {@code id} at once on the screen rectangle (x1, y1)-(x2, y2) with texture
     * coordinates (u1, v1)-(u2, v2) in [0, 1]. Flushes the GUI buffer first, so what was queued
     * before lies beneath.
     */
    static void texture(GuiGraphics g, ResourceLocation id, double x1, double y1, double x2, double y2,
                        float u1, float v1, float u2, float v2) {
        g.flush();
        RenderSystem.setShaderTexture(0, id);
        RenderSystem.setShader(GameRenderer::getPositionTexShader);
        RenderSystem.enableBlend();
        RenderSystem.defaultBlendFunc();
        Matrix4f m = g.pose().last().pose();
        BufferBuilder bb = Tesselator.getInstance().begin(VertexFormat.Mode.QUADS, DefaultVertexFormat.POSITION_TEX);
        bb.addVertex(m, (float) x1, (float) y1, 0).setUv(u1, v1);
        bb.addVertex(m, (float) x1, (float) y2, 0).setUv(u1, v2);
        bb.addVertex(m, (float) x2, (float) y2, 0).setUv(u2, v2);
        bb.addVertex(m, (float) x2, (float) y1, 0).setUv(u2, v1);
        BufferUploader.drawWithShader(bb.buildOrThrow());
        RenderSystem.disableBlend();
    }
}
