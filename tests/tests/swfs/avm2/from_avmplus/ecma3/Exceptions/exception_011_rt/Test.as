/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


// un-caught exception
// Undefined throw test.

var thisError = "Exited with uncaught exception";

try {
    throw (void 0);
} catch(e:Error) {
    thisError = e.toString();
    trace("FAILED!: Should have exited with uncaught exception.");
} finally {
    Assert.expectEq("Thrown undefined should be uncaught.", "Exited with uncaught exception", thisError);
}
