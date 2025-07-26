/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import avmplus.System;
import flash.utils.ByteArray;
import com.adobe.test.Assert;

// var SECTION = "Array";
// var VERSION = "AS3";
// var TITLE   = "Array methods that handle/mangle length";
// var bug     = "661330";


// Array
var azero;
var afive;
var amax0;
var amax2;
var amax5;

// Subclass
var testing_subclass:Boolean = true;
var szero;
var sfive;
var smax0;
var smax2;
var smax5;

// Vector
var testing_vector:Boolean = true;
var vzero;
var vfive;
var vmax0;
var vmax2;
var vmax5;

// ByteArray
var testing_bytearray:Boolean = true;
var bzero;
var bfive;
var bmax0;
var bmax2;
var bmax5;

// Object (with length property)
var testing_objwithlength:Boolean = true;
var ozero;
var ofive;
var omax0;
var omax2;
var omax5;

// This is the constructor for the Object cases above
function WithLength(l) { this.length = l; }

var all_arrays;    // array alternating strings and arrays of array subclasses.
var all_arraylike; // array alternating strings and arrays of array-likes.

function apply_arrays(f) {
    var i = 1;
    all_arrays[1].map(f); i += 2; // array
    if (i >= all_arrays.length) return;
    all_arrays[3].map(f); i += 2; // subclass of array
}

// Maps f over every odd-indexed element of all_arraylike (thus f is
// mapped over the array-likes).
function apply_alikes(f) {
    // (unrolled for loop so that stack-traces say which arraylike is faulting)
    var i = 1;
    all_arraylike[1].map(f); i += 2; // array
    if (i >= all_arraylike.length) return;
    all_arraylike[3].map(f); i += 2; // subclass of array
    if (i >= all_arraylike.length) return;
    all_arraylike[5].map(f); i += 2; // vector
    if (i >= all_arraylike.length) return;
    all_arraylike[7].map(f); i += 2; // bytearray
    if (i >= all_arraylike.length) return;
    all_arraylike[9].map(f); i += 2; // object with length
}

function reset() {
    all_arrays    = [];
    all_arraylike = [];

    function newarrays(name)
    {
        all_arrays.push(name);
        all_arrays.push([]);
        newalikes(name);
    }

    function newalikes(name)
    {
        all_arraylike.push(name);
        all_arraylike.push([]);
    }

    function pushlike(a)
    {
        var top = all_arraylike[all_arraylike.length - 1];
        top.push(a);
        return a;
    }

    function pusharray(a)
    {
        var top = all_arrays[all_arrays.length - 1];
        top.push(a);
        return pushlike(a);
    }

    newarrays("arrays:");
    azero = pusharray(new Array(0));
    afive = pusharray(new Array(5));
    amax0 = pusharray(new Array(uint.MAX_VALUE));
    amax2 = pusharray(new Array(uint.MAX_VALUE - 2));
    amax5 = pusharray(new Array(uint.MAX_VALUE - 5));

    if (testing_subclass)
    {
        newarrays("subarrays:");
        ozero = pusharray(new Subarray(0));
        ofive = pusharray(new Subarray(5));
        omax0 = pusharray(new Subarray(uint.MAX_VALUE));
        omax2 = pusharray(new Subarray(uint.MAX_VALUE - 2));
        omax5 = pusharray(new Subarray(uint.MAX_VALUE - 5));
    }

    if (testing_vector)
    {
        newalikes("vectors:");
        vzero = pushlike(new Vector.<int>(0));
        vfive = pushlike(new Vector.<int>(5));
        // These cause assertion failures in Debug (not necessarily
        // expected; see Bugzilla 685520) and probably would cause out
        // of memory failures if uncommented anyway.
        // vmax0 = pushlike(new Vector.<int>(uint.MAX_VALUE));
        // vmax2 = pushlike(new Vector.<int>(uint.MAX_VALUE - 2));
        // vmax5 = pushlike(new Vector.<int>(uint.MAX_VALUE - 5));
    }

    if (testing_bytearray)
    {
        newalikes("bytearrays:");
        bzero = pushlike(new ByteArray()); bzero.length = 0;
        bfive = pushlike(new ByteArray()); bfive.length = 5;
        // These cause system to run out of memory (as expected).
        //bmax0 = pushlike(new ByteArray()); bmax0.length = uint.MAX_VALUE;
        //bmax2 = pushlike(new ByteArray()); bmax2.length = uint.MAX_VALUE - 2;
        //bmax5 = pushlike(new ByteArray()); bmax5.length = uint.MAX_VALUE - 5;
    }

    if (testing_objwithlength)
    {
        newalikes("objects:");
        ozero = pushlike(new WithLength(0));
        ofive = pushlike(new WithLength(5));
        omax0 = pushlike(new WithLength(uint.MAX_VALUE));
        omax2 = pushlike(new WithLength(uint.MAX_VALUE - 2));
        omax5 = pushlike(new WithLength(uint.MAX_VALUE - 5));
    }
}

function alternating_lengths(arr)
{
    var result = [];
    for (var i=0; i < arr.length; i+=2) {
        var name = arr[i];
        var arraylikes = arr[i+1];
        result.push(name);
        result = result.concat(arraylikes.map(function (a) {return a.length;}))
    }
    return result.toString();
}

function arrays_len() {
    return alternating_lengths(all_arrays);
}
function alikes_len() {
    return alternating_lengths(all_arraylike);
}

function test_arrays(testname, fcn,
                     expected_array_lengths,
                     expected_subarray_lengths) {
    reset();
    apply_arrays(fcn);
    var expected =
        (expected_array_lengths+
         (testing_subclass ? ","+expected_subarray_lengths : ""));
    Assert.expectEq(testname, expected, arrays_len());
}

function test_arraylikes(testname, fcn,
                         expected_array_lengths,
                         expected_subarray_lengths,
                         expected_vector_lengths,
                         expected_bytearray_lengths,
                         expected_objwithlength_lengths) {
    reset();
    apply_alikes(fcn);
    var expected =
        (expected_array_lengths+
         (testing_subclass ? ","+expected_subarray_lengths : "")+
         (testing_vector ? ","+expected_vector_lengths : "")+
         (testing_bytearray ? ","+expected_bytearray_lengths : "")+
         (testing_objwithlength ? ","+expected_objwithlength_lengths : ""));
    Assert.expectEq(testname, expected, alikes_len());
}


// For ease of comparison in actual tests below, here are lengths of all arrays.
test_arrays("base line", function (a) { /*no-op*/ },
            "arrays:,0,5,4294967295,4294967293,4294967290",
            "subarrays:,0,5,4294967295,4294967293,4294967290");

// Analogously, base line lengths for all the array-likes.
test_arraylikes("base line", function(a) { /*no-op*/ },
                "arrays:,0,5,4294967295,4294967293,4294967290",
                "subarrays:,0,5,4294967295,4294967293,4294967290",
                "vectors:,0,5",
                "bytearrays:,0,5",
                "objects:,0,5,4294967295,4294967293,4294967290");

test_arrays("auto small", function(a) { a[5] = 5; },
            "arrays:,6,6,4294967295,4294967293,4294967290",
            "subarrays:,6,6,4294967295,4294967293,4294967290");

test_arrays("auto large", function(a) { a[5] = 5; a[1000] = 1000; },
            "arrays:,1001,1001,4294967295,4294967293,4294967290",
            "subarrays:,1001,1001,4294967295,4294967293,4294967290");

test_arrays("length direct 1K", function(a) { a.length = 1000; },
            "arrays:,1000,1000,1000,1000,1000",
            "subarrays:,1000,1000,1000,1000,1000");

test_arrays("length indirect 1K", function(a) { a["length"] = 1000 },
            "arrays:,1000,1000,1000,1000,1000",
            "subarrays:,1000,1000,1000,1000,1000");

test_arrays("length direct wrap", function(a) { a.length = uint.MAX_VALUE+2; },
            "arrays:,1,1,1,1,1",
            "subarrays:,1,1,1,1,1");

test_arrays("length indirect wrap", function(a) { a["length"] = uint.MAX_VALUE+2; },
            "arrays:,1,1,1,1,1",
            "subarrays:,1,1,1,1,1");

test_arrays("pub pop arrays", function(a) { a.public::pop(); },
            "arrays:,0,4,4294967294,4294967292,4294967289",
            "subarrays:,0,4,4294967294,4294967292,4294967289");

test_arrays("AS3 pop arrays", function(a) { a.AS3::pop(); },
            "arrays:,0,4,4294967294,4294967292,4294967289",
            "subarrays:,0,4,4294967294,4294967292,4294967289");

// Need to qualify pop as public::pop so that it will resolve
// to the generic (and non-bound) pop method.
test_arraylikes("pub pop alikes",
                function(a) { Array.prototype.public::pop.call(a); },
                "arrays:,0,4,4294967294,4294967292,4294967289",
                "subarrays:,0,4,4294967294,4294967292,4294967289",
                "vectors:,0,4",
                "bytearrays:,0,4",
                "objects:,0,4,4294967294,4294967292,4294967289");



// uncomment (and remove "false ||") when Bugzilla 681803 fixed
if (false /*|| System.swfVersion >= 15*/)
{
    test_arrays("pub push arrays", function(a) { a.public::push(1,2,3,4,5); },
                "arrays:,5,10,4294967295,4294967295,4294967295",
                "subarrays:,5,10,4294967295,4294967295,4294967295");
}
else
{
    test_arrays("pub push arrays", function(a) { a.public::push(1,2,3,4,5); },
                "arrays:,5,10,4,2,4294967295",
                "subarrays:,5,10,4,2,4294967295");
}

test_arrays("AS3 push arrays", function(a) { a.AS3::push(1,2,3,4,5); },
            "arrays:,5,10,4294967295,4294967295,4294967295",
            "subarrays:,5,10,4294967295,4294967295,4294967295");

// uncomment (and remove "false ||") when Bugzilla 681803 fixed
if (false /*|| System.swfVersion >= 15*/)
{
    // Need to qualify pop as public::push so that it will resolve
    // to the generic (and non-bound) pop method.

    test_arraylikes("pub push alikes",
                    function(a) { Array.prototype.public::push.call(a,1,2,3,4,5); },
                    "arrays:,5,10,4294967295,4294967295,4294967295",
                    "subarrays:,5,10,4294967295,4294967295,4294967295",
                    "vectors:,5,10",
                    "bytearrays:,5,10",
                    "objects:,5,10,4294967295,4294967295,4294967295");
}
else
{
    test_arraylikes("pub push alikes",
                    function(a) { Array.prototype.public::push.call(a,1,2,3,4,5); },
                    "arrays:,5,10,4,2,4294967295",
                    "subarrays:,5,10,4,2,4294967295",
                    "vectors:,5,10",
                    "bytearrays:,5,10",
                    "objects:,5,10,4,2,4294967295");
}



test_arrays("pub shift arrays", function(a) { a.public::shift(); },
            "arrays:,0,4,4294967294,4294967292,4294967289",
            "subarrays:,0,4,4294967294,4294967292,4294967289");

test_arrays("AS3 shift arrays", function(a) { a.AS3::shift(); },
            "arrays:,0,4,4294967294,4294967292,4294967289",
            "subarrays:,0,4,4294967294,4294967292,4294967289");

test_arraylikes("pub shift alikes",
                function(a) {

                    // The generic shift takes too long to run (and
                    // occupies excess amount of memory) for the
                    // generic object case when the object's .length
                    // is anywhere near uint.MAX_VALUE.
                    //
                    // So we do not bother for the large Object cases.

                    if (a instanceof WithLength && a.length > 5)
                    {
                        /* no-op */
                    }
                    else
                    {
                        Array.prototype.public::shift.call(a);
                    }
                },
                "arrays:,0,4,4294967294,4294967292,4294967289",
                "subarrays:,0,4,4294967294,4294967292,4294967289",
                "vectors:,0,4",
                "bytearrays:,0,4",
                "objects:,0,4,4294967295,4294967293,4294967290");


// uncomment (and remove "false ||") when Bugzilla 685323 fixed
if (false /*|| System.swfVersion >= 15*/)
{
    test_arrays("pub unshift arrays", function(a) { a.public::unshift(1,2,3,4,5); },
                "arrays:,5,10,4294967295,4294967295,4294967295",
                "subarrays:,5,10,4294967295,4294967295,4294967295");
}
else
{
    test_arrays("pub unshift arrays", function(a) { a.public::unshift(1,2,3,4,5); },
                "arrays:,5,10,4,2,4294967295",
                "subarrays:,5,10,4,2,4294967295")
}

// uncomment (and remove "false ||") when Bugzilla 685323 fixed
if (false /*|| System.swfVersion >= 15*/)
{
    test_arrays("AS3 unshift arrays", function(a) { a.AS3::unshift(1,2,3,4,5); },
                "arrays:,5,10,4294967295,4294967295,4294967295",
                "subarrays:,5,10,4294967295,4294967295,4294967295")
}
else
{
    test_arrays("AS3 unshift arrays", function(a) { a.AS3::unshift(1,2,3,4,5); },
                "arrays:,5,10,4,2,4294967295",
                "subarrays:,5,10,4,2,4294967295")
}

test_arraylikes("unshift alikes",
                function(a) {

                    // Unsurprisingly, the generic unshift takes too
                    // long to run for the generic object case when
                    // the object's .length is anywhere near
                    // uint.MAX_VALUE, making it currently infeasible
                    // to observe wrap-around via unshift.  So we do
                    // not bother for the large Object cases.

                    if (a instanceof WithLength && a.length > 5)
                    {
                        /* no-op */
                    }
                    else if (a is Vector.<int>)
                    {
                        // The generic var-args Array unshift expands
                        // too fast for Vector (which cannot be
                        // helped).  Unshifting by one element is an
                        // accomodation for Vector.

                        Array.prototype.public::unshift.call(a,5);
                        Array.prototype.public::unshift.call(a,4);
                        Array.prototype.public::unshift.call(a,3);
                        Array.prototype.public::unshift.call(a,2);
                        Array.prototype.public::unshift.call(a,1);
                    }
                    else
                    {
                        Array.prototype.public::unshift.call(a,1,2,3,4,5);
                    }
                },
                // uncomment when Bugzilla 685323 fixed
                ((false /*|| System.swfVersion >= 15*/)
                 ? "arrays:,5,10,4294967295,4294967295,4294967295"
                 : "arrays:,5,10,4,2,4294967295"),
                ((false /*|| System.swfVersion >= 15*/)
                 ? "subarrays:,5,10,4294967295,4294967295,4294967295"
                 : "subarrays:,5,10,4,2,4294967295"),
                "vectors:,5,10",
                "bytearrays:,5,10",
                // See note above.
                "objects:,5,10,4294967295,4294967293,4294967290")
