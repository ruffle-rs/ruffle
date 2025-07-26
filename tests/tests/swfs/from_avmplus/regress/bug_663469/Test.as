/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import com.adobe.test.Assert;

function sampleOutOfRangeTest(... _args)
{
    // sample a variety of integers that are out of range, *and*
    // too large to fit into kIntptrType atom on 32-bit builds.
    // (Some may happen to not crash, depending on memory layout,
    // but all are accessing undefined memory.)
    var idx:int;
    idx = 0x1fffffff; trace("foo ",idx," is ",_args[idx]);
    idx = 0x2fffffff; trace("foo ",idx," is ",_args[idx]);
    idx = 0x7fffffff; trace("foo ",idx," is ",_args[idx]);
    idx = 0xdeadbeef; trace("foo ",idx," is ",_args[idx]);
    idx = 0x5DCD64BA; trace("foo ",idx," is ",_args[idx]);
}
function sampleOutOfRange()
{
    // must call test() from a jitted method
    for (var i:int=0;i<10;i++) {
        sampleOutOfRangeTest();
    }
    return true;
}

Assert.expectEq("sample variety of integers out of range", true, sampleOutOfRange());

