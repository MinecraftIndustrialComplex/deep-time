// SPDX-License-Identifier: LGPL-3.0-or-later
/**
 * Deep Time's public climate API (P1-25): the simulated climate of a Deep Time world per block —
 * monthly temperatures at the block's height, annual mean, warmest and coldest month, precipitation
 * and its seasonality, the Köppen class — and, since API version 2, the planet's geometry on both
 * sides (circumference and latitude; mic-climate's hemisphere seasons colour leaves by it on the
 * client). Start at {@link
 * com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate}; versioned by {@link
 * com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate#API_VERSION}. Only
 * {@code DeepTimeClimate} and {@code ClimateSample} are API; everything else here is internal.
 */
package com.minecraftindustrialcomplex.deeptime.api.climate;
