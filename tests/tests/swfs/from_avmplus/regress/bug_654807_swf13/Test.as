/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "regress_654807";
// var VERSION = "AS3";
// var TITLE   = "SealedArray";
// var bug = "654807";



    // standard Assert.expectError function doesn't handle no-error well.
    function MyTestCase(desc:String, expected:String, testFunc:Function)
    {
        try {
            var actual = testFunc();
            Assert.expectEq(desc, expected, String(actual));
        } catch (e) {
            actualErr = e;
            Utils.grabError(actualErr, expected);
            Assert.expectEq(desc, expected, String(actualErr).substr(0, expected.length));
        }
    }

    class SealedArray extends Array {}
    dynamic class DynamicArray extends Array {}
    dynamic class DynamicSealedArray extends SealedArray {}
    class SealedDynamicArray extends DynamicArray {}

    function run_tests(b:Array, mode:String)
    {
        // idiom "String(b.concat())" is used to create a copy of the
        // possibly-subclass-of-Array, since toString may not work on the subclass

        MyTestCase(mode+" get_length",
            "0",
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" push",
            { dynamic:"0,1,2", semisealed:"0,1,2", sealed:"ReferenceError: Error #1056"}[mode],
            function() { b.push(0,1,2); return String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"3", semisealed:"3", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" unshift",
            { dynamic:"4,5,6,0,1,2", semisealed:"4,5,6,0,1,2", sealed:"ReferenceError: Error #1056"}[mode],
            function() { b.unshift(4,5,6); return String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"6", semisealed:"6", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" concat",
            { dynamic:"4,5,6,0,1,2,4,5,6,0,1,2", semisealed:"4,5,6,0,1,2,4,5,6,0,1,2", sealed:""}[mode],
            function() { var r = b.concat(b); return String(r.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"6", semisealed:"6", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" reverse",
            { dynamic:"2,1,0,6,5,4", semisealed:"2,1,0,6,5,4", sealed:""}[mode],
            function() { b.reverse(); return String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"6", semisealed:"6", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" shift",
            { dynamic:"2:1,0,6,5,4", semisealed:"2:1,0,6,5,4", sealed:"undefined:"}[mode],
            function() { var r = b.shift(); return String(r) + ":" + String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"5", semisealed:"5", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        // splice fails differently when delCount == 0 vs delCount > 0
        MyTestCase(mode+" splice",
            { dynamic:"9,1,0,6,5,4", semisealed:"9,1,0,6,5,4", sealed:"ReferenceError: Error #1056"}[mode],
            function() { b.splice(0, 0, 9); return String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"6", semisealed:"6", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" pop",
            { dynamic:"4:9,1,0,6,5", semisealed:"4:9,1,0,6,5", sealed:"undefined:"}[mode],
            function() { var r = b.pop(); return String(r) + ":" + String(b.concat()); });

        MyTestCase(mode+" get_length",
            { dynamic:"5", semisealed:"5", sealed:"0"}[mode],
            function() { var r = b.length; return String(r); });

        MyTestCase(mode+" set_length",
            { dynamic:"3:9,1,0", semisealed:"5:9,1,0,6,5", sealed:"0:"}[mode],
            function() { b.length = 3; var r = b.length; return String(r) + ":" + String(b.concat()); });

        MyTestCase(mode+" getprop",
            { dynamic:"9", semisealed:"ReferenceError: Error #1069", sealed:"ReferenceError: Error #1069"}[mode],
            function() { return b[0]; });

        MyTestCase(mode+" setprop",
            { dynamic:"undefined", semisealed:"ReferenceError: Error #1056", sealed:"ReferenceError: Error #1056"}[mode],
            function() { b[0] = 44; });

        MyTestCase(mode+" hasprop",
            { dynamic:"true", semisealed:"false", sealed:"false"}[mode],
            function() { return b.hasOwnProperty(0); });

        MyTestCase(mode+" delprop",
            { dynamic:"true", semisealed:"false", sealed:"false"}[mode],
            function() { return delete b[0]; });

        // splice fails differently when delCount == 0 vs delCount > 0
        MyTestCase(mode+" splice",
            { dynamic:"9,1,0", semisealed:"ReferenceError: Error #1069", sealed:"ReferenceError: Error #1056"}[mode],
            function() { b.splice(0, 1, 9); return String(b.concat()); });

        MyTestCase(mode+" join",
            { dynamic:"9~1~0", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return String(b.join("~")); });

        MyTestCase(mode+" toString",
            { dynamic:"9,1,0", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return b.toString(); });

        MyTestCase(mode+" toLocaleString",
            { dynamic:"9,1,0", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return b.toLocaleString(); });

        MyTestCase(mode+" sort",
            { dynamic:"0,1,9", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return String(b.sort(Array.NUMERIC)); });

        MyTestCase(mode+" indexOf",
            { dynamic:"2", semisealed:"ReferenceError: Error #1069", sealed:"-1"}[mode],
            function() { return String(b.indexOf(9)); });

        MyTestCase(mode+" lastIndexOf",
            { dynamic:"2", semisealed:"ReferenceError: Error #1069", sealed:"-1"}[mode],
            function() { return String(b.lastIndexOf(9)); });

        MyTestCase(mode+" every",
            { dynamic:"true", semisealed:"ReferenceError: Error #1069", sealed:"true"}[mode],
            function() { return String(b.every( function(i) { return true; } )); });

        MyTestCase(mode+" filter",
            { dynamic:"0,9", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return String(b.filter( function(i) { return i != 1; } )); });

        MyTestCase(mode+" forEach",
            { dynamic:"undefined", semisealed:"ReferenceError: Error #1069", sealed:"undefined"}[mode],
            function() { return String(b.forEach( function(i) { } )); });

        MyTestCase(mode+" some",
            { dynamic:"true", semisealed:"ReferenceError: Error #1069", sealed:"false"}[mode],
            function() { return String(b.some( function(i) { return true } )); });

        MyTestCase(mode+" map",
            { dynamic:"1,2,10", semisealed:"ReferenceError: Error #1069", sealed:""}[mode],
            function() { return String(b.map( function(i) { return i+1 } )); });

        MyTestCase(mode+" for...in",
            { dynamic:"0,1,2,", semisealed:"0,1,2,3,4,", sealed:""}[mode],
            function() { var s = ""; for (var i in b) { s += String(i) + ","; } return s; });

        MyTestCase(mode+" for each...in",
            { dynamic:"0,1,9,", semisealed:"9,1,0,6,5,", sealed:""}[mode],
            function() { var s = ""; for each (var i in b) { s += String(i) + ","; } return s; });

    }
    run_tests(new Array, "dynamic");
    run_tests(new SealedArray, false ? "semisealed" : "sealed");
    run_tests(new DynamicArray, "dynamic");
    run_tests(new DynamicSealedArray, "dynamic");
    run_tests(new SealedDynamicArray, false ? "semisealed" : "sealed");


