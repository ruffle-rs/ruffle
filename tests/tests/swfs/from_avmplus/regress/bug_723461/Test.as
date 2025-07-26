/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


// var SECTION = "regress_723461";
// var VERSION = "AS3";
// var TITLE   = "ByteArray readUTFBytes does not properly update position when it starts at a UTF8 BOM";
// var bug = "723461";


import flash.utils.ByteArray;
import com.adobe.test.Assert;
function testCore(starting) {
    var results = {};
    var starting = 0;
    var b:ByteArray = new ByteArray();
    b.position = starting;
    b.writeUTFBytes('\ufeff1234');
    var posPostWrite = b.position;
    var len = b.length;
    b.writeUTFBytes('5678');
    var s1;
    var s2;
    var posPostRead;

    b.position = starting;
    try {
        s1 = b.readUTFBytes(len);
    } catch (e) { }
    posPostRead = b.position;

    try {
        s2 = b.readUTFBytes(4);
    } catch (e) { }


    results.s1 = s1;
    results.s2 = s2;
    results.posPostWrite = posPostWrite;
    results.posPostRead = posPostRead;

    return results;
}

function testStartAt0() {
    var results = testCore(0);

    Assert.expectEq("at 0 result string skips BOM",
                "1234",
                results.s1);

    Assert.expectEq("at 0 result subsequent reads correct",
                "5678",
                results.s2);

    Assert.expectEq("at 0 position updated correctly",
                results.posPostWrite,
                results.posPostRead);
}

function testStartAt10() {
    var results = testCore(10);

    Assert.expectEq("at 10 result string skips BOM",
                "1234",
                results.s1);

    Assert.expectEq("at 10 result subsequent reads correct",
                "5678",
                results.s2);

    Assert.expectEq("at 10 position updated correctly",
                results.posPostWrite,
                results.posPostRead);
}

testStartAt0();

testStartAt10();

