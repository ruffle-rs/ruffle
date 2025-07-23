/* -*- Mode: js; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}
// (used logicalAssignment.as as template for this code)

// var SECTION = "Expressions";
// var VERSION = "AS3";
// var TITLE   = "Index Property Boundary Cases";

// This code tests that set/get/in/delete all "behave", for a variety
// of integer indices on a variety of indexable maps.
//
// Integer indices tested:
// - every case of exactly one set bit:   0*10* (ie 2^k, for 0 <= k < 64)
// - every case of exactly one bit unset: 1*01*
// - every case of zero-run then one-run: 0*1*  (ie 2^k-1, for 0<=k<64)
// - every case of one-run then zero-run: 1*0*
//
// Indexable maps tested:
// Object, Array, ByteArray, and Vector


import flash.utils.ByteArray;

import com.adobe.test.Assert;
class MapProps {
    var name                 // string naming type of map being tested
    var numtestbits          // number of distinct bits to flip in tests
                             // (doubled for maps supporting negative indices)
    var min_index            // minimum i for which we test m[i]
    var max_index            // maximum i for which we test m[i]; always 2^n-1
    var get_throws_if_absent // if (i in m) false then m[i] throws exn, ...
    var defaultval_if_absent // and o/w, if (i in m) false then this is m[i]
    var supports_delete      // delete m[i] "works"; subsequent (i in m) false

    var initially_absent     // true iff type treats unset as initially absent

    function testname(desc, idx) {
        return (desc+", "+this.name+" ["+idx+"]");
    }

    function MapProps( name,
                       numtestbits          = 32,
                       min_index            = undefined, // -2^#bits; see below
                       max_index            = undefined, // 2^#bits-1; see below
                       get_throws_if_absent = false,
                       defaultval_if_absent = undefined,
                       supports_delete      = true,
                       initially_absent     = true)
    {
        this.name                 = name
        this.numtestbits          = numtestbits
        this.min_index            =
            (min_index !== undefined) ? min_index : - Math.pow(2, numtestbits);
        this.max_index            =
            (max_index !== undefined) ? max_index : Math.pow(2, numtestbits)-1;
        this.get_throws_if_absent = get_throws_if_absent
        this.defaultval_if_absent = defaultval_if_absent
        this.supports_delete      = supports_delete
        this.initially_absent     = initially_absent
    }
};

class ByteArrayProps extends MapProps
{
    function ByteArrayProps(name, numtestbits, min_index, max_index) {
        super(name,
              numtestbits,
              min_index,
              max_index,
              /* get_throws_if_absent */ false,
              /* defaultval_if_absent */ 0,
              /* supports_delete      */ false,
              /* initially_absent     */ false);

        // N.B.: Setting initially_absent to false for ByteArray
        // because while entries outside of the largest set index are
        // considered absent, entries less than the largest set index
        // are implicitly present (and I believe set to 0).  At one
        // time I had more complex logic to deal with such
        // "auto-filling", but at this point I think it just belongs
        // in a ByteArray-specific set of tests, which is not exactly
        // the point of this set of tests.
    }
}

class VectorProps extends MapProps
{
    function VectorProps(name, numtestbits, min_index, max_index) {
        super(name,
              numtestbits,
              min_index,
              max_index,
              /* get_throws_if_absent */ true,
              /* defaultval_if_absent */ undefined,
              /* supports_delete      */ false,
              /* initially_absent     */ false);
    }
}

// Helper for testing exception signaling
// @return true iff thunk() throws exception
function check_throws(thunk) {
    var threw = false;
    try {
        thunk();
    } catch (e) {
        threw = true;
    }
    return threw;
}

// map: map-instance being tested
// props: meta-data about map (see MapProps class above)
// idx: index being tested
// idxfamily: string describing class of idx
// absent: boolean flag, true iff map[idx] is unset + not automatically filled
// newval: value to set map[idx] to in test.
// @return true if all tests succeeded.
function test_one_index(map, props, idx, idxfamily, absent, newval) {

    // N.B. the commented out test cases are for benefit of someone
    // inspecting failures in future, to narrow down which case caused
    // success flag to be set to false.  So the intention here is that
    // the success flag tracks whether all attempted tests for this
    // index have passed; this procedure always returns the success
    // flag, but can also provide more specifics after manual hacking.

    var success = true; // assume success until proven otherwise.
    var name;
    var result;

    // test lookups when initially absent.
    if (absent) {
        result = ((idx in map) == false);
        name = props.testname(idxfamily+" idx in map, initially absent", idx);
        // Assert.expectEq(name, true, result);
        success &&= result;

        if (props.get_throws_if_absent) {
            name = props.testname(idxfamily+" map[idx] throws if absent", idx);
            result = check_throws(function() { return map[idx] });
            // Assert.expectEq(name, true, result);
            success &&= result;
        } else {
            name = props.testname(idxfamily+" map[idx] default if absent", idx);
            result = (props.defaultval_if_absent == map[idx]);
            // Assert.expectEq(name, true, result);
            success &&= result;
        }
    }

    // Now do the actual assignment.
    map[idx] = newval;

    name = props.testname(idxfamily+" idx in map after assignment", idx);
    result = idx in map;
    // Assert.expectEq(name, true, result);
    success &&= result;
    name = props.testname(idxfamily+" map[idx] after assignment", idx);
    result = (newval == map[idx]);
    // Assert.expectEq(name, true, result);
    success &&= result;

    if (props.supports_delete) {
        // Now check that delete removes the element
        delete map[idx];

        name = props.testname(idxfamily+" idx in map, absent post-delete", idx);
        result = (false == (idx in map));
        // Assert.expectEq(name, true, result);
        if (props.get_throws_if_absent) {
            name = props.testname(idxfamily+" map[idx] throw post delete", idx);
            result = check_throws(function() { return map[idx] });
            // Assert.expectEq(name, true, result);
            success &&= result;
        } else {
            name = props.testname(idxfamily+" map[idx] deflt post delete", idx);
            result = (props.defaultval_if_absent == map[idx]);
            // Assert.expectEq(name, true, result);
            success &&= result;
        }
    }

    return success;
}

function guarded_test_one_index( map, props, idx, idxfamily, absent, newval ) {
    if (idx >= props.min_index && idx <= props.max_index)
        return test_one_index(map, props, idx, idxfamily, absent, newval);
    else
        return true;
}

// N.B.: the for-loops below deliberately strive to go from largest
// index down to smallest; decided to do this after encountering
// ByteArray's auto-fill behavior (the testing of which I later
// abandoned).  Going from smallest up to largest would not catch
// auto-fill cases.

function test_onebitset( map, props, absent ) { // aka 0*10*
    var success = true; // assume success until proven otherwise
    for (var i = props.numtestbits-1; i >= 0; i--) {
        var idx = Math.pow(2, i)
        success =
            (guarded_test_one_index( map, props, idx, "onebitset", absent, 1 )
             && success);
    }
    Assert.expectEq(props.name+" onebitset", true, success);
}

function test_onebitoff( map, props, absent ) { // aka 1*01*
    var success = true; // assume success until proven otherwise
    for (var i = 0; i < props.numtestbits; i++) {
        var idx = props.max_index - Math.pow(2, i);
        success =
            (guarded_test_one_index( map, props, idx, "onebitoff", absent, 2 )
             && success);
    }
    Assert.expectEq(props.name+" onebitoff", true, success);
}

function test_runzerone( map, props, absent ) { // aka 0*1*
    var success = true; // assume success until proven otherwise
    for (var i = props.numtestbits-1; i >= 0; i--) {
        var idx = Math.pow(2, i) - 1
        success =
            (guarded_test_one_index( map, props, idx, "runzerone", absent, 3 )
             && success);
    }
    Assert.expectEq(props.name+" runzerone", true, success);
}

function test_runonezer( map, props, absent ) { // aka 1*0*
    var success = true; // assume success until proven otherwise
    for (var i = 0; i < props.numtestbits; i++) {
        var idx = props.max_index - (Math.pow(2, i) - 1);
        success =
            (guarded_test_one_index( map, props, idx, "runonezer", absent, 4 )
             && success);
    }
    Assert.expectEq(props.name+" runonezer", true, success);
}

// Runs each of the above test familys, each family with a fresh map.
// (avoids dealing with overlapping indexes between families, which
//  would complicate the absent-logic in the test; this way we just
//  assume map is fresh and so presence is dictated by map-type, not
//  past test-modifications to the map instance.)
// mapMaker: constructs empty instance of map type being tested.
function composed_test(mapMaker, props) {
    test_onebitset(mapMaker(), props, props.initially_absent)
    test_onebitoff(mapMaker(), props, props.initially_absent)
    test_runzerone(mapMaker(), props, props.initially_absent)
    test_runonezer(mapMaker(), props, props.initially_absent)
}

var twoPow64 = Math.pow(2, 64);

// object
var objMakerLit = function () { return {}; };
composed_test(objMakerLit, new MapProps("{}"));
var objMakerNew = function () { return new Object(); };
composed_test(objMakerNew, new MapProps("new Object()"));

// array
var arrMakerLit = function () { return []; };
composed_test(arrMakerLit, new MapProps("[]"));
var arrMakerNew = function () { return new Array(); };
composed_test(arrMakerNew, new MapProps("new Array()"));

// ByteArray
// On 32-bit build, this runs out of memory for large 2^i
// (e.g. i>=30 unworkable); but must cover *some* i >= 23, since
// bug 559082 specifically found a problem for 2^23.
//
// On a Motorola Droid phone, the kernel's OOM process-killer
// SIGKILL's the avmshell when bytArrBits > 25, so that's an
// (inclusive) upper-bound for now.
//
// And even on a 64-bit build, i>=30 seems unworkable:
//
// EXPONENT    TIME (MacBook Pro, 2.8 Ghz Core 2 Duo, 4 GB ram
// bytArrBits  median 3 runs 64-bit release build)
//       30      84.1 sec
//       29       3.1 sec
//       28       2.1 sec
//       27       1.1 sec
//       26       1.1 sec
//       25       1.1 sec
//       24       1.1 sec
// (The blow-up going up from 27 above may be symptom of performance bug,
//  perhaps bug 556023.)

var bytArrBits = 25;
var bytArrLen = Math.pow(2, bytArrBits)-1;

var bytArrMaker = function () { return new ByteArray(); };
var bytearray_props = new ByteArrayProps("new ByteArray()", bytArrBits, 0, bytArrLen);
composed_test(bytArrMaker, bytearray_props);

// prior experiments indicated that system gets overly sluggish for
// vector of length >= 2^27 > 134,000,000; which *is* a lot of memory
// to initialize.  Even 2^22 increases the test running time by a full
// second compared to 2^21 (and 2^20).  So that partially influences
// choice of vectorBits value below.

var vectorBits = 20;
var vecLen = Math.pow(2, vectorBits)-1;

var vecAnyMaker = function () { return new Vector.<*>(vecLen, /*fixed=*/false); };
var vecAny_props = new VectorProps("new Vector.<*>", vectorBits, 0, vecLen);
composed_test(vecAnyMaker, vecAny_props);

var vecIntMaker = function () { return new Vector.<int>(vecLen, /*fixed=*/false); };
var vecInt_props = new VectorProps("new Vector.<int>", vectorBits, 0, vecLen);
composed_test(vecIntMaker, vecInt_props);

var veUintMaker = function () { return new Vector.<uint>(vecLen, /*fixed=*/false); };
var veUint_props = new VectorProps("new Vector.<uint>", vectorBits, 0, vecLen);
composed_test(veUintMaker, veUint_props);

var vecNumMaker = function () { return new Vector.<Number>(vecLen, /*fixed=*/false); };
var vecNum_props = new VectorProps("new Vector.<Number>", vectorBits, 0, vecLen);
composed_test(vecAnyMaker, vecNum_props);
