// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import java.util.Optional;
import java.util.OptionalLong;

/**
 * How a {@link Site} is mined (plan §5.6); the COE and CDG hooks (P1-16) read it.
 *
 * @param finite      whether the site runs out
 * @param amountBase  the base amount (ore tonnes today; P1-16 scales it to gameplay amounts)
 * @param coeVein     the Create Ore Excavation vein recipe id the site feeds, if any
 * @param cdgChargeMb the Create Diesel Generators charge in millibuckets, if any
 */
public record Extraction(boolean finite, double amountBase, Optional<String> coeVein, OptionalLong cdgChargeMb) {}
