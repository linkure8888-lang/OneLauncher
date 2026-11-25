package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.phys.EntityHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec3;

import java.util.concurrent.ThreadLocalRandom;

/**
 * Hit particles: a coloured burst at the point where the swing landed. Critical hits get a
 * brighter, slightly larger burst plus vanilla crit sparks.
 *
 * <p>Particles are spawned client-side only, so the effect is purely local and nothing extra is
 * sent to the server.</p>
 */
final class RemixHitEffects {

    /** Gold, so a crit is readable even when the configured colour is close to it. */
    private static final int CRIT_COLOR = 0xE0B84A;
    private static final float NORMAL_SCALE = 0.9f;
    private static final float CRIT_SCALE = 1.25f;

    private RemixHitEffects() {
    }

    /**
     * Called before the local player attacks. {@code attackStrengthScale} filters click spam out:
     * without it a fast clicker would stack a burst per click instead of one per real swing.
     */
    static void onAttack(Minecraft client, LocalPlayer player, RemixConfig config) {
        if (!config.hitEffects || client.level == null || player == null) {
            return;
        }
        if (player.getAttackStrengthScale(0.5f) < 0.9f) {
            return;
        }

        HitResult hit = client.hitResult;
        if (!(hit instanceof EntityHitResult entityHit)
                || !(entityHit.getEntity() instanceof LivingEntity)) {
            return;
        }

        Vec3 point = entityHit.getLocation();
        if (point == null) {
            return;
        }
        // The local player's own crit condition, without the sprint and blindness checks:
        // those need state the client does not always have for the swing that just happened.
        boolean crit = player.fallDistance > 0.0 && !player.onGround();
        burst(client.level, point, crit ? CRIT_COLOR : config.hitParticleColor,
                config.hitParticleCount, crit);
    }

    static void burst(ClientLevel level, Vec3 point, int rgb, int count, boolean crit) {
        DustParticleOptions dust = new DustParticleOptions(rgb, crit ? CRIT_SCALE : NORMAL_SCALE);
        ThreadLocalRandom random = ThreadLocalRandom.current();

        for (int i = 0; i < count; i++) {
            double angle = Math.PI * 2.0 * i / count;
            // An outward ring plus a little upward lift reads as a "bubble" at the hit point,
            // which is what players expect from this effect.
            double vx = Math.cos(angle) * 0.18 + random.nextDouble(-0.02, 0.02);
            double vy = 0.10 + random.nextDouble(0.0, 0.08);
            double vz = Math.sin(angle) * 0.18 + random.nextDouble(-0.02, 0.02);
            level.addParticle(dust, point.x, point.y, point.z, vx, vy, vz);
        }

        if (crit) {
            level.addParticle(ParticleTypes.CRIT, point.x, point.y, point.z, 0.0, 0.0, 0.0);
        }
    }
}
