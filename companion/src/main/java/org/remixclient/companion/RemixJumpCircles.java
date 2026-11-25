package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.world.phys.Vec3;

/**
 * Jump circles: a short particle ring around the feet every time the player leaves the ground.
 *
 * <p>Purely client-side, and it only fires on an upward jump — falling off a block does not
 * trigger it, so a ring always means "I jumped".</p>
 */
final class RemixJumpCircles {

    private static final int POINTS = 14;
    private static final double RADIUS = 0.55;
    private static final int COLOR = 0x7C5CFF;

    private static boolean wasOnGround;

    private RemixJumpCircles() {
    }

    static void tick(Minecraft client, RemixConfig config) {
        LocalPlayer player = client.player;
        if (!config.jumpCircles || player == null || client.level == null) {
            wasOnGround = false;
            return;
        }

        boolean onGround = player.onGround();
        boolean jumpedUp = !onGround && player.getDeltaMovement().y > 0.1;
        if (wasOnGround && jumpedUp) {
            ring(client, player);
        }
        wasOnGround = onGround;
    }

    private static void ring(Minecraft client, LocalPlayer player) {
        Vec3 feet = new Vec3(player.getX(), player.getY() + 0.02, player.getZ());
        for (int i = 0; i < POINTS; i++) {
            double angle = Math.PI * 2.0 * i / POINTS;
            double x = feet.x + Math.cos(angle) * RADIUS;
            double z = feet.z + Math.sin(angle) * RADIUS;
            // Tiny outward drift so the ring expands instead of sitting still.
            client.level.addParticle(new DustParticleOptions(COLOR, 0.8f),
                    x, feet.y, z, Math.cos(angle) * 0.02, 0.0, Math.sin(angle) * 0.02);
        }
    }
}
