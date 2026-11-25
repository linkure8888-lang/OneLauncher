package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.world.phys.Vec3;

/**
 * A soft particle trail behind the player while walking.
 *
 * <p>Purely client-side: the particles live in the local level, so servers see nothing extra.
 * Spawns are spaced by distance walked, not by time, so standing still draws nothing and a
 * sprint keeps the density steady.</p>
 */
final class RemixTrails {

    private static final double SPACING = 0.45;
    private static final int COLOR = 0x7C5CFF;

    /** Feet position of the previous spawn; null until the first tick on the ground. */
    private static Vec3 lastSpawn;

    private RemixTrails() {
    }

    static void tick(Minecraft client, RemixConfig config) {
        LocalPlayer player = client.player;
        if (!config.trails || player == null || client.level == null) {
            lastSpawn = null;
            return;
        }
        try {
            if (!player.onGround() || player.getDeltaMovement().lengthSqr() < 1.0E-4) {
                // Airborne or standing still: keep the anchor so the first step draws.
                return;
            }

            Vec3 feet = player.position();
            if (lastSpawn == null || feet.distanceTo(lastSpawn) >= SPACING) {
                spawn(client, feet);
                lastSpawn = feet;
            }
        } catch (Throwable t) {
            RemixCompanionClient.LOGGER.error("[Remix Companion] Trail tick failed.", t);
            lastSpawn = null;
        }
    }

    static void reset() {
        lastSpawn = null;
    }

    private static void spawn(Minecraft client, Vec3 feet) {
        // Two staggered puffs with a slight upward drift, like the vanilla portal ambience.
        client.level.addParticle(new DustParticleOptions(COLOR, 0.9f),
                feet.x, feet.y + 0.05, feet.z, 0.0, 0.03, 0.0);
        client.level.addParticle(new DustParticleOptions(COLOR, 0.6f),
                feet.x + 0.1, feet.y + 0.12, feet.z + 0.1, 0.0, 0.01, 0.0);
    }
}
