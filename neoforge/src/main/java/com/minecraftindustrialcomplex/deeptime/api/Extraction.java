// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import java.util.Optional;
import java.util.OptionalLong;

/**
 * How a {@link Site} is mined (plan §5.6); the COE and CDG hooks (P1-16) read it.
 *
 * @param finite      whether the site runs out
 * @param amountBase  the base amount (ore tonnes, the physical value)
 * @param coeVein     the Create Ore Excavation vein recipe id the site was written with, if any (the
 *                    COE hook reads the deposit type's current {@code coe_vein} first; P1-16)
 * @param cdgChargeMb the physical oil volume in millibuckets, if any (CDG sees it × the type's
 *                    {@code cdg_scale}, split over the ore body's chunks; P1-16)
 */
public record Extraction(boolean finite, double amountBase, Optional<String> coeVein, OptionalLong cdgChargeMb) {}
