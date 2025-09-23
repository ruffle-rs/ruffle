/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

function test_pos()
{
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
        if (String(d) != String(d*1.0))
            print(i + " " + d + " " + d*1.0)
    }
    return i;
}

function test_neg()
{
    // d starts life as a kIntegerType atom
    var d = -65535
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
        if (String(d) != String(d*1.0))
            print(i + " " + d + " " + d*1.0)
    }
    return i;
}


    var results = []

    var e = (-1 >>> 0)
    results.push({expected: 4294967295, actual: e});

    results.push({expected: 2147483648, actual: (- - "0x80000000")});

    results.push({expected: 64, actual: test_pos()});
    results.push({expected: 64, actual: test_neg()});

    var u:uint = 0xffffffff;
    results.push({expected: 0, actual: ~u});
    results.push({expected: -4294967295, actual: -u});

for (var i in results)
{
    var o = results[i]
    Assert.expectEq("test_"+i, o.expected, o.actual);
}
