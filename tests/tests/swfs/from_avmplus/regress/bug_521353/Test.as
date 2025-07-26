/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

function f() {
    // d starts life as a kIntegerType atom
    var d = 0xffff|0
    var i = 16

    // double d over and over to exceed the range limits of atom and 32 bit,
    // to try to demonstrate extra precision that shouldn't be there.  To test
    // for precision, we explicitly convert d to double by multiplication, then
    // mask of the low bits and compare with an unconverted (masked) d.
    //
    // if d has more precision than double supports, the masked result should
    // be different.

    while ((d&15) == ((d*1.0)&15) && i < 64) {
        d = d + d + 1
        i++
        print(i + " " + d + " " + d*1.0)
    }
    if (i == 64) {
        // d's precision stayed the same
        return "pass"
    } else {
        // d's kIntegerType precision exceed that of kDoubleType
        return "fail"
    }
}


Assert.expectEq('Bug 521353 - optimized fast path for OP_add in Interpreter preserves too much precision on 64bit cpu', 'pass', f());

