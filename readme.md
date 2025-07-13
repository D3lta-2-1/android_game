# This is personal experimentation on small physic simulations in Rust.

This repository contains a dozen of global physics solvers, each one try to get me closer to the saintly grail of a perfect physics engine.
And by that I mean energy conservation, stability and flexibility.

## Presentation of each solver :
- ### First Order
  It's the most basic impulse solver, close to the first solvers used by Box2d in 2006, except that, like all solvers, use an exact solution instead of an iterative one.
  Without any kind of stabilization, important drift can be observed, especially with large time steps.
- ### Second Order
  This solver is a forced based solver, following David Baraff and Andrew Witkin's work on the subject. It looks very promising, but it's eaten alive by integration errors.
  Using Verlet integration instead of implicit Euler integration make things worse.
- ### HybridV1 - aka - First Order with Prepass
  This solver solve first on forces, then on impulses. My hope was that by chaining solvers, I could get the best of both worlds (except the performance penalty).
  The integration step is done with a second order Taylor expansion, which is a bit more stable than the first order one.
  This solver is slow but have way less drift, however it's still affected by energy conservation issues.
- ### HybridV2
  I thought finding with second order solver with corrected velocities could improve the calculated acceleration. Turn out it doesn't.
  This solver is probably the worst of all of them, most of the time things explode, and when they don't, they drift like hell.
  I kept it because it makes the other solvers look good and for the sake of completeness.
- ### HybridV3
  This solver is another attempt to combine the first order and second order solvers, but this time in one step.
  Using Taylor expansion and some form of early integration, it's able to solve on both forces and impulses together, which is lighter than the HybridV1 and HybridV2.
  When it comes to raw accuracy, it's the best solver I made so far. It has a good enough energy conservation without any kind of stabilization.
- ### HybridV3cgm
  Same as HybridV3, but it uses the CGM (Conjugate Gradient Method) to solve the linear system. Provide the same results as HybridV3.
- ### HybridV4
  Since going to the second order improved things so much, I thought I could try to go to the third order. Turn out it's useless and got me headaches.
- ### Pbd - aka - Position Based Dynamics
  Until now, I've only experimented on solving at force and impulse level, but this time I tried to solve at position level.
  I didn't read any paper on the subject and only experimented with the idea. I believe this is what PBD do, so I named it like that. Correct me if I'm wrong.
  Because of its nature, drift is inexistant, but it has a lot of issues with energy conservation.
  I believe I'm able to get exact solutions because I only work with positions, but taking rotations into account will make it non-linear, aka with no exact solution.
- ### HybridV3Pbd
  This solver is a combination of the HybridV3 and the PBD. It uses the HybridV3 to get the forces and impulses, then it uses the PBD to solve the positions.
  I feel like it's what Rapier have done for a long time before switching to a TGS solver.
- ### FirstOrderSoft
  Same as first order, but with a soft constraint. That means stabilization.
  On most scenes, stiffness is set to infinity, which make it equivalent to Baumgarte stabilization.
- ### HybridV3Soft
  Same as HybridV3, but with a soft constraint. That means stabilization.
  On most scenes, stiffness is set to infinity, which make it equivalent to Baumgarte stabilization. For reason that are beyond my understanding, this solver can be a bit explosive.
## Desktop Build :
```cargo run --package desktop```
or 
```cargo run```

## Android Build :
This project can be built for Android, for testing math coherency on different architectures.

```./gradlew assembleDebug```
you can also use android studio or IntelliJ IDEA to build the project.

warning: you need to have android SDK and NDK installed on your machine.  
You're likely to change the path to the SDK and NDK in the build.gradle file.