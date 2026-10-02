/* SPDX-License-Identifier: LGPL-3.0-or-later
 *
 * Reference harness for crates/deeptime-noddy (plan P2-01): calls the ORIGINAL Noddy functions
 * (Loop3D/noddyverse src/unEvents.c, taste.c, utils.c, ellFault.c, proflib.c; MIT) on random
 * events and points and prints what they did, for the Rust port's tests/reference.rs to compare
 * against. Built and run by tools/validation/noddy/reference.sh, which clones noddyverse at the
 * pinned commit; no Noddy source is vendored here.
 *
 * Output, one case after another:
 *   case <n>
 *   event fold <px> <py> <pz> <dipdir> <dip> <pitch> <wavelength> <amplitude> <cycle> <single>
 *   event fault <kind> <geometry> <movement> <px> <py> <pz> <dipdir> <dip> <pitch> <rotation>
 *               <slip> <amplitude> <radius> <width> <xaxis> <yaxis> <zaxis> <cylindex> <profpitch>
 *   event unc <px> <py> <pz> <dipdir> <dip>
 *   event dyke <kind> <merge> <px> <py> <pz> <dipdir> <dip> <slippitch> <sliplength> <width>
 *   event plug <shape> <merge> <px> <py> <pz> <dipdir> <dip> <axispitch> <radius> <apical>
 *              <bvalue> <a> <b> <c>
 *   event tilt <px> <py> <pz> <rotation> <plungedir> <plunge>
 *   event strain <9 entries of the inverse tensor, row-major>
 *   points <n>
 *   <x> <y> <z> <x0> <y0> <z0> <again> <16 hex digits: sequence bytes 0-7> <flavor> <index>
 *   end
 * Enumerations are Noddy's own integers (nodStruc.h): fault kind FAULT_EVENT 0, SHEAR_ZONE_EVENT
 * 1, KINK_EVENT 2; geometry TRANSLATION 0, ROTATION 1, ELLIPTICAL 2, CURVED 3, RING 4; movement
 * HANGING_WALL 0, FOOT_WALL 1, BOTH 2; dyke DILATION_DYKE 0, STOPE_DYKE 1; plug CYLINDRICAL_PLUG
 * 0, CONE_PLUG 1, PARABOLIC_PLUG 2, ELLIPSOIDAL_PLUG 3. Events are listed oldest first; the event
 * list the original sees is a base stratigraphy (index 0) followed by them.
 *
 * Each event's orientation is set up the way getEvent.c's loaders do it (convrt, rotset, plane,
 * paxis, irotate), with the loaders' special cases (a vertical fault is 89.999 degrees, a vertical
 * dyke's dip direction turns by 180, a plug's frame is built from dip - 90, ...).
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "xvt.h"
#include "noddy.h"

#define MAX_EVENTS 16
#define NPOINTS 40

static OBJECT objects[MAX_EVENTS + 1];
static int numObjects = 0;

/* The event list the original walks (nodLib*.c in the real program). */
WINDOW getEventDrawingWindow () { return (WINDOW) 0; }
int countObjects (WINDOW win) { return numObjects; }
OBJECT *nthObject (WINDOW win, int n)
{
   if (n < 0 || n >= numObjects) return NULL;
   return &objects[n];
}
int incrementLongJob (int n) { return 1; }

static unsigned long long rngState = 0x9E3779B97F4A7C15ULL;
static double rnd (void)
{
   rngState = rngState * 6364136223846793005ULL + 1442695040888963407ULL;
   return (double) (rngState >> 11) / 9007199254740992.0;
}
static double uni (double lo, double hi) { return lo + (hi - lo) * rnd (); }
static int pick (int n) { int k = (int) (rnd () * n); return k >= n ? n - 1 : k; }

static float *sineTable (void)
{
   static float *t = NULL;
   int i;
   if (!t)
   {
      t = (float *) malloc (sizeof (float) * (X_PROFILE_RESOLUTION + 1));
      for (i = 0; i <= X_PROFILE_RESOLUTION; i++)
         t[i] = (float) (100.0 * sin (6.283185307179586 * (double) i / 628.0));
   }
   return t;
}

static void position (double *x, double *y, double *z)
{
   *x = uni (-1500.0, 1500.0);
   *y = uni (-1500.0, 1500.0);
   *z = uni (-1500.0, 1500.0);
}

static void addFold (int single, int cylindrical)
{
   FOLD_OPTIONS *o = (FOLD_OPTIONS *) calloc (1, sizeof (FOLD_OPTIONS));
   o->type = SINE_FOLD;
   o->singleFold = single;
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->dipDirection = uni (0.0, 360.0);
   o->dip = uni (20.0, 90.0);
   o->axisPitch = uni (0.0, 180.0);
   o->wavelength = uni (500.0, 3000.0);
   o->amplitude = uni (50.0, 500.0);
   o->cycle = cylindrical ? 0.0 : uni (500.0, 3000.0);
   o->profile.array = sineTable ();
   convrt (o->dip, o->dipDirection, o->axisPitch, TRUE);
   rotset (o->dip, o->dipDirection, o->axisPitch,
           o->rotationMatrix.forward, o->rotationMatrix.reverse);
   objects[numObjects].shape = FOLD;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event fold %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %d\n",
           o->positionX, o->positionY, o->positionZ, o->dipDirection, o->dip, o->axisPitch,
           o->wavelength, o->amplitude, o->cycle, o->singleFold);
}

static void addFault (int kind, int geometry, int movement)
{
   FAULT_OPTIONS *o = (FAULT_OPTIONS *) calloc (1, sizeof (FAULT_OPTIONS));
   double pitch, dip;
   o->type = kind;
   o->geometry = geometry;
   o->movement = movement;
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->dipDirection = uni (0.0, 360.0);
   o->dip = pick (6) == 0 ? 90.0 : uni (15.0, 89.0);
   o->pitch = uni (0.0, 180.0);
   o->rotation = uni (-40.0, 40.0);
   o->slip = uni (-600.0, 600.0);
   o->amplitude = uni (50.0, 400.0);
   o->radius = uni (500.0, 2000.0);
   o->width = uni (100.0, 800.0);
   o->xAxis = uni (1500.0, 4000.0);
   o->yAxis = uni (1500.0, 4000.0);
   o->zAxis = uni (1500.0, 4000.0);
   o->cylindricalIndex = pick (3) == 0 ? 0.0 : uni (500.0, 3000.0);
   o->profilePitch = uni (0.0, 180.0);
   o->profile.array = sineTable ();
   o->surfaceType = FLAT_SURFACE;
   if (geometry == CURVED || geometry == ELLIPTICAL)
      o->slip = uni (-0.2, 0.2) * o->xAxis;   /* the original's solver needs a modest slip */
   pitch = (o->geometry == ROTATION) ? 0.0 : o->pitch;
   dip = (o->dip == 90.0) ? 89.999 : o->dip;
   convrt (dip, o->dipDirection, pitch, TRUE);
   rotset (dip, o->dipDirection, pitch, o->rotationMatrix.forward, o->rotationMatrix.reverse);
   plane (&o->plane.a, &o->plane.b, &o->plane.c, &o->plane.d,
          o->positionX, o->positionY, o->positionZ);
   objects[numObjects].shape = (kind == FAULT_EVENT) ? FAULT : SHEAR_ZONE;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event fault %d %d %d %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g\n",
           kind, geometry, movement, o->positionX, o->positionY, o->positionZ, o->dipDirection,
           o->dip, o->pitch, o->rotation, o->slip, o->amplitude, o->radius, o->width, o->xAxis,
           o->yAxis, o->zAxis, o->cylindricalIndex, o->profilePitch);
}

static void addUnconformity (void)
{
   UNCONFORMITY_OPTIONS *o = (UNCONFORMITY_OPTIONS *) calloc (1, sizeof (UNCONFORMITY_OPTIONS));
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->dipDirection = uni (0.0, 360.0);
   o->dip = uni (0.0, 40.0);
   o->surfaceType = FLAT_SURFACE;
   convrt (o->dip, o->dipDirection, 0.0, TRUE);
   plane (&o->plane.a, &o->plane.b, &o->plane.c, &o->plane.d,
          o->positionX, o->positionY, o->positionZ);
   rotset (o->dip, o->dipDirection, 0.0, o->rotationMatrix.forward, o->rotationMatrix.reverse);
   objects[numObjects].shape = UNCONFORMITY;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event unc %.17g %.17g %.17g %.17g %.17g\n",
           o->positionX, o->positionY, o->positionZ, o->dipDirection, o->dip);
}

static void addDyke (int kind, int merge)
{
   DYKE_OPTIONS *o = (DYKE_OPTIONS *) calloc (1, sizeof (DYKE_OPTIONS));
   double dipDirection;
   o->type = kind;
   o->mergeEvents = merge;
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->dipDirection = uni (0.0, 360.0);
   o->dip = pick (4) == 0 ? 90.0 : uni (30.0, 89.0);
   o->slipPitch = uni (0.0, 180.0);
   o->slipLength = uni (-300.0, 300.0);
   o->width = uni (100.0, 900.0);
   dipDirection = (o->dip == 90.0) ? o->dipDirection + 180.0 : o->dipDirection;
   convrt (o->dip, dipDirection, 90.0, TRUE);
   rotset (o->dip, dipDirection, 90.0, o->rotationMatrix.forward, o->rotationMatrix.reverse);
   objects[numObjects].shape = DYKE;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event dyke %d %d %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g\n",
           kind, merge, o->positionX, o->positionY, o->positionZ, o->dipDirection, o->dip,
           o->slipPitch, o->slipLength, o->width);
}

static void addPlug (int shape, int merge)
{
   PLUG_OPTIONS *o = (PLUG_OPTIONS *) calloc (1, sizeof (PLUG_OPTIONS));
   double pitch;
   o->type = shape;
   o->mergeEvents = merge;
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->dipDirection = uni (0.0, 360.0);
   o->dip = uni (30.0, 150.0);
   o->axisPitch = uni (0.0, 180.0);
   o->radius = uni (400.0, 1800.0);
   o->apicalAngle = uni (10.0, 70.0);
   o->BValue = uni (100.0, 2000.0);
   o->axisA = uni (500.0, 2500.0);
   o->axisB = uni (500.0, 2500.0);
   o->axisC = uni (500.0, 2500.0);
   pitch = (o->type != ELLIPSOIDAL_PLUG) ? 0.0 : o->axisPitch;
   convrt (o->dip - 90.0, o->dipDirection, pitch, TRUE);
   rotset (o->dip - 90.0, o->dipDirection, pitch,
           o->rotationMatrix.forward, o->rotationMatrix.reverse);
   objects[numObjects].shape = PLUG;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event plug %d %d %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g\n",
           shape, merge, o->positionX, o->positionY, o->positionZ, o->dipDirection, o->dip,
           o->axisPitch, o->radius, o->apicalAngle, o->BValue, o->axisA, o->axisB, o->axisC);
}

static void addTilt (void)
{
   TILT_OPTIONS *o = (TILT_OPTIONS *) calloc (1, sizeof (TILT_OPTIONS));
   double axis[2][4], mata[4][4], angle;
   int i, j;
   position (&o->positionX, &o->positionY, &o->positionZ);
   o->plunge = uni (0.0, 60.0);
   o->plungeDirection = uni (0.0, 360.0);
   o->rotation = uni (-60.0, 60.0);
   angle = o->rotation * 0.0174532;
   paxis (o->plunge, o->plungeDirection, axis);
   irotate (mata, axis[1][1], axis[1][2], axis[1][3], angle);
   for (i = 0; i < 3; i++)
      for (j = 0; j < 3; j++)
         o->rotationMatrix[i][j] = mata[i + 1][j + 1];
   objects[numObjects].shape = TILT;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event tilt %.17g %.17g %.17g %.17g %.17g %.17g\n",
           o->positionX, o->positionY, o->positionZ, o->rotation, o->plungeDirection, o->plunge);
}

static void addStrain (void)
{
   STRAIN_OPTIONS *o = (STRAIN_OPTIONS *) calloc (1, sizeof (STRAIN_OPTIONS));
   int i, j;
   for (i = 0; i < 3; i++)
      for (j = 0; j < 3; j++)
         o->inverseTensor[i][j] = (i == j ? uni (0.6, 1.6) : uni (-0.3, 0.3));
   objects[numObjects].shape = STRAIN;
   objects[numObjects].options = (char *) o;
   numObjects++;
   printf ("event strain");
   for (i = 0; i < 3; i++)
      for (j = 0; j < 3; j++)
         printf (" %.17g", o->inverseTensor[i][j]);
   printf ("\n");
}

static void addRandom (void)
{
   switch (pick (13))
   {
      case 0: addFold (0, 1); break;
      case 1: addFold (pick (2), 0); break;
      case 2: addFault (FAULT_EVENT, TRANSLATION, pick (3)); break;
      case 3: addFault (FAULT_EVENT, ROTATION, pick (3)); break;
      case 4: addFault (FAULT_EVENT, ELLIPTICAL, pick (3)); break;
      case 5: addFault (FAULT_EVENT, CURVED, pick (3)); break;
      case 6: addFault (FAULT_EVENT, RING, pick (3)); break;
      case 7: addFault (1 + pick (2), pick (2) ? RING : (pick (2) ? ROTATION : TRANSLATION), pick (3)); break;
      case 8: addUnconformity (); break;
      case 9: addDyke (pick (2), pick (2)); break;
      case 10: addPlug (pick (4), pick (2)); break;
      case 11: addTilt (); break;
      default: addStrain (); break;
   }
}

static void beginCase (int n)
{
   int i;
   for (i = 1; i < numObjects; i++)
      free (objects[i].options);
   memset (objects, 0, sizeof (objects));
   objects[0].shape = STRATIGRAPHY;
   numObjects = 1;
   printf ("case %d\n", n);
}

static void runCase (void)
{
   double ***dots;
   STORY **histoire;
   double in[NPOINTS + 1][3];
   int m, k;
   unsigned int flavor;
   int index;

   dots = (double ***) malloc (sizeof (double **) * (NPOINTS + 1));
   histoire = (STORY **) malloc (sizeof (STORY *) * (NPOINTS + 1));
   for (m = 0; m <= NPOINTS; m++)
   {
      dots[m] = (double **) malloc (sizeof (double *) * 2);
      dots[m][0] = (double *) calloc (4, sizeof (double));
      dots[m][1] = (double *) calloc (4, sizeof (double));
      histoire[m] = (STORY *) calloc (2, sizeof (STORY));
   }
   for (m = 1; m <= NPOINTS; m++)
   {
      for (k = 0; k < 3; k++)
      {
         in[m][k] = uni (-3000.0, 3000.0);
         dots[m][1][k + 1] = in[m][k];
      }
      histoire[m][1].again = TRUE;
      izero (histoire[m][1].sequence);
   }
   reverseEvents (dots, histoire, NPOINTS, 1);
   printf ("points %d\n", NPOINTS);
   for (m = 1; m <= NPOINTS; m++)
   {
      taste (numObjects, histoire[m][1].sequence, &flavor, &index);
      printf ("%.17g %.17g %.17g %.17g %.17g %.17g %d ", in[m][0], in[m][1], in[m][2],
              dots[m][1][1], dots[m][1][2], dots[m][1][3], histoire[m][1].again ? 1 : 0);
      for (k = 0; k < 8; k++)
         printf ("%02x", histoire[m][1].sequence[k]);
      printf (" %u %d\n", flavor, index);
   }
   printf ("end\n");
   for (m = 0; m <= NPOINTS; m++)
   {
      free (dots[m][0]);
      free (dots[m][1]);
      free (dots[m]);
      free (histoire[m]);
   }
   free (dots);
   free (histoire);
}

int main (int argc, char *argv[])
{
   int n = 0, rep, g, mv, k, events;

   printf ("# noddy reference: original unEvents.c, taste.c, utils.c, ellFault.c, proflib.c, nodLib3.c\n");
   /* One event at a time, every kind. */
   for (rep = 0; rep < 3; rep++)
   {
      beginCase (n++); addFold (0, 1); runCase ();
      beginCase (n++); addFold (0, 0); runCase ();
      beginCase (n++); addFold (1, rep % 2); runCase ();
      for (g = TRANSLATION; g <= RING; g++)
         for (mv = HANGING_WALL; mv <= BOTH; mv++)
         {
            beginCase (n++); addFault (FAULT_EVENT, g, mv); runCase ();
         }
      for (k = SHEAR_ZONE_EVENT; k <= KINK_EVENT; k++)
      {
         beginCase (n++); addFault (k, TRANSLATION, pick (3)); runCase ();
         beginCase (n++); addFault (k, ROTATION, pick (3)); runCase ();
         beginCase (n++); addFault (k, RING, pick (3)); runCase ();
      }
      beginCase (n++); addUnconformity (); runCase ();
      beginCase (n++); addDyke (DILATION_DYKE, 0); runCase ();
      beginCase (n++); addDyke (STOPE_DYKE, 0); runCase ();
      for (k = CYLINDRICAL_PLUG; k <= ELLIPSOIDAL_PLUG; k++)
      {
         beginCase (n++); addPlug (k, 0); runCase ();
      }
      beginCase (n++); addTilt (); runCase ();
      beginCase (n++); addStrain (); runCase ();
   }
   /* Histories: order of composition, stops at unconformities and intrusions, merged igneous
   ** events, taste over faults. */
   for (rep = 0; rep < 60; rep++)
   {
      beginCase (n++);
      events = 2 + pick (6);
      for (k = 0; k < events; k++)
         addRandom ();
      runCase ();
   }
   return 0;
}
