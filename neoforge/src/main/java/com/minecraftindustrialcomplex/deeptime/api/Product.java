// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import net.minecraft.resources.ResourceLocation;

/**
 * One product of a {@link Site} (plan §5.6; FORMAT.md "Deposit sites").
 *
 * @param material  a {@code c:} material id ({@code c:copper}); its ore blocks are {@code #c:ores/<name>}
 * @param grade     mass fraction of the material in the ore (1 for a product that is the whole ore)
 * @param tonnageT  ore tonnes the grade applies to (contained metal = tonnage × grade)
 * @param role      {@code primary}, {@code byproduct}, …
 */
public record Product(ResourceLocation material, float grade, double tonnageT, String role) {}
