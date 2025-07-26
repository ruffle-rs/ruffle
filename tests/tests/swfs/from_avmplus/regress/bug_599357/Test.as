/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=599357
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Utils;
import com.adobe.test.Assert;
// var SECTION = "599357";
// var VERSION = "";
// var TITLE   = "Testing indexed get/set prop access with numeric expressions";
// var bug = "599357";

var testcases = getTestCases();

    function getArrayAddII(a:Array, b:int, c:int){
        try {
            return (a[(b + c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getArrayIntToNumber(_arg1:Array, _arg2:int){
        var arr:Number = _arg2;
        return (_arg1[arr]);
    };
    function getArrayUintToNumber(_arg1:Array, _arg2:uint){
        var _local3:Number = _arg2;
        return (_arg1[_local3]);
    };
    function getArrayConstToNumber(_arg1:Array){
        var _local2:Number = 1;
        return (_arg1[_local2]);
    };
    function getArrayAddIC(a:Array, b:int){
        try {
            return (a[(b + 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getArrayAddCI(a:Array, b:int){
        try {
            return (a[(1 + b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getArraySubII(a:Array, b:int, c:int){
        try {
            return (a[(b - c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getArraySubIC(a:Array, b:int){
        try {
            return (a[(b - 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getArraySubCI(a:Array, b:int){
        try {
            return (a[(1 - b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorAddII(a:Vector.<int>, b:int, c:int){
        try {
            return (a[(b + c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorAddIC(a:Vector.<int>, b:int){
        try {
            return (a[(b + 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorAddCI(a:Vector.<int>, b:int){
        try {
            return (a[(1 + b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorSubII(a:Vector.<int>, b:int, c:int){
        try {
            return (a[(b - c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorSubIC(a:Vector.<int>, b:int){
        try {
            return (a[(b - 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function getVectorSubCI(a:Vector.<int>, b:int){
        try {
            return (a[(1 - b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArrayAddII(a:Array, b:int, c:int, d:int){
        try {
            a[(b + c)] = d;
            return (a[(b + c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArrayIntToNumber(a:Array, b:int, c:int){
        var d:Number = b;
        a[d] = c;
        return (a[d]);
    };
    function setArrayUintToNumber(a:Array, b:uint, c:int){
        var d:Number = b;
        a[d] = c;
        return (a[d]);
    };
    function setArrayConstToNumber(a:Array, b:int){
        var d:Number = 1;
        a[d] = b;
        return (a[d]);
    };
    function setArrayAddIC(a:Array, b:int, d:int){
        try {
            a[(b + 1)] = d;
            return (a[(b + 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArrayAddCI(a:Array, b:int, d:int){
        try {
            a[(1 + b)] = d;
            return (a[(1 + b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArraySubII(a:Array, b:int, c:int, d:int){
        try {
            a[(b - c)] = d;
            return (a[(b - c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArraySubIC(a:Array, b:int, d:int){
        try {
            a[(b - 1)] = d;
            return (a[(b - 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setArraySubCI(a:Array, b:int, d:int){
        try {
            a[(1 - b)] = d;
            return (a[(1 - b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorAddII(a:Vector.<int>, b:int, c:int, d:int){
        try {
            a[(b + c)] = d;
            return (a[(b + c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorAddIC(a:Vector.<int>, b:int, d:int){
        try {
            a[(b + 1)] = d;
            return (a[(b + 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorAddCI(a:Vector.<int>, b:int, d:int){
        try {
            a[(1 + b)] = d;
            return (a[(1 + b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorSubII(a:Vector.<int>, b:int, c:int, d:int){
        try {
            a[(b - c)] = d;
            return (a[(b - c)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorSubIC(a:Vector.<int>, b:int, d:int){
        try {
            a[(b - 1)] = d;
            return (a[(b - 1)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };
    function setVectorSubCI(a:Vector.<int>, b:int, d:int){
        try {
            a[(1 - b)] = d;
            return (a[(1 - b)]);
        } catch(e) {
            return (Utils.grabError(e, e.toString()));
        };
    };


    function getTestCases() {
        var array:Array = new Array();
        var index:int = 0;
        var arr:Array = [1, 2, 3, 4, 5];
        var vec:Vector.<int> = new Vector.<int>();
        vec.fixed = false;
        vec[0] = 1;
        vec[1] = 2;
        vec[2] = 3;
        vec[3] = 4;
        vec[4] = 5;
        array[index++] = Assert.expectEq( "array[1+2]", 4, getArrayAddII(arr, 1, 2));
        array[index++] = Assert.expectEq( "array[1+ -2]", undefined, getArrayAddII(arr, 1, -2));
        array[index++] = Assert.expectEq( "array[0x7fffffff+0x7fffffff]", undefined, getArrayAddII(arr, 0x7fffffff, 0x7fffffff));
        array[index++] = Assert.expectEq( "array[0x7fffffff+ -0x7fffffff]", 1, getArrayAddII(arr, 0x7fffffff, -0x7fffffff));
        array[index++] = Assert.expectEq( "array[-0x7fffffff+ -0x7fffffff]", undefined, getArrayAddII(arr, -0x7fffffff, -0x7fffffff));
        array[index++] = Assert.expectEq( "array[2+1], where 1 is const", 4, getArrayAddIC(arr, 2));
        array[index++] = Assert.expectEq( "array[2-1], where 1 is const", 2, getArraySubIC(arr, 2));
        array[index++] = Assert.expectEq( "array[1+2], where 1 is const", 4, getArrayAddCI(arr, 2));
        array[index++] = Assert.expectEq( "array[1-2], where 1 is const", undefined, getArraySubCI(arr, 2));
        array[index++] = Assert.expectEq( "array[2-1]", 2, getArraySubII(arr, 2, 1));
        array[index++] = Assert.expectEq( "array[0x80000000-0x80000000]", 1, getArraySubII(arr, 0x80000000, 0x80000000));
        array[index++] = Assert.expectEq( "array[0x80000000+0x80000000]", undefined, getArrayAddII(arr, 0x80000000, 0x80000000));

        array[index++] = Assert.expectEq( "vector[1+2]", 4, getVectorAddII(vec, 1, 2));
        array[index++] = Assert.expectEq( "vector[1+ -2]", "Error #1125", getVectorAddII(vec, 1, -2));
        array[index++] = Assert.expectEq( "vector[0x7fffffff+0x7fffffff]", "Error #1125", getVectorAddII(vec, 0x7fffffff, 0x7fffffff));
        array[index++] = Assert.expectEq( "vector[0x7fffffff+ -0x7fffffff]", 1, getVectorAddII(vec, 0x7fffffff, -0x7fffffff));
        array[index++] = Assert.expectEq( "vector[-0x7fffffff+ -0x7fffffff]", "Error #1125", getVectorAddII(vec, -0x7fffffff, -0x7fffffff));
        array[index++] = Assert.expectEq( "vector[2+1], where 1 is const", 4, getVectorAddIC(vec, 2));
        array[index++] = Assert.expectEq( "vector[2-1], where 1 is const", 2, getVectorSubIC(vec, 2));
        array[index++] = Assert.expectEq( "vector[1+2], where 1 is const", 4, getVectorAddCI(vec, 2));
        array[index++] = Assert.expectEq( "vector[1-2], where 1 is const", "Error #1125", getVectorSubCI(vec, 2));
        array[index++] = Assert.expectEq( "vector[2-1]", 2, getVectorSubII(vec, 2, 1));
        array[index++] = Assert.expectEq( "vector[0x80000000-0x80000000]", 1, getVectorSubII(vec, 0x80000000, 0x80000000));
        array[index++] = Assert.expectEq( "vector[0x80000000+0x80000000]", "Error #1125", getVectorAddII(vec, 0x80000000, 0x80000000));

        array[index++] = Assert.expectEq( "array[1], where 1 is a const stored in a Number var", 2, getArrayConstToNumber(arr));
        array[index++] = Assert.expectEq( "array[1], where 1 is a int stored in a Number var", 2, getArrayIntToNumber(arr, 1));
        array[index++] = Assert.expectEq( "array[1], where 1 is a uint stored in a Number var", 2, getArrayUintToNumber(arr, 1));

        array[index++] = Assert.expectEq( "set array[1], where 1 is a const stored in a Number var", 100, setArrayConstToNumber(arr, 100));
        array[index++] = Assert.expectEq( "set array[1], where 1 is a int stored in a Number var", 101, setArrayIntToNumber(arr, 1, 101));
        array[index++] = Assert.expectEq( "set array[1], where 1 is a uint stored in a Number var", 102, setArrayUintToNumber(arr, 1, 102));

        array[index++] = Assert.expectEq( "set array[1+2]", 10, setArrayAddII(arr, 1, 2, 10));
        array[index++] = Assert.expectEq( "set array[1+ -2]", 11, setArrayAddII(arr, 1, -2, 11));
        array[index++] = Assert.expectEq( "set array[0x7fffffff+0x7fffffff]", 12, setArrayAddII(arr, 0x7fffffff, 0x7fffffff, 12));
        array[index++] = Assert.expectEq( "set array[0x7fffffff+ -0x7fffffff]", 13, setArrayAddII(arr, 0x7fffffff, -0x7fffffff, 13));
        array[index++] = Assert.expectEq( "set array[-0x7fffffff+ -0x7fffffff]", 14, setArrayAddII(arr, -0x7fffffff, -0x7fffffff, 14));
        array[index++] = Assert.expectEq( "set array[2+1], where 1 is const", 15, setArrayAddIC(arr, 2, 15));
        array[index++] = Assert.expectEq( "set array[1+2], where 1 is const", 17, setArrayAddCI(arr, 2, 17));
        array[index++] = Assert.expectEq( "set array[2-1], where 1 is const", 16, setArraySubIC(arr, 2, 16));
        array[index++] = Assert.expectEq( "set array[1-2], where 1 is const", 18, setArraySubCI(arr, 2, 18));
        array[index++] = Assert.expectEq( "set array[2-1]", 19, setArraySubII(arr, 2, 1, 19));
        array[index++] = Assert.expectEq( "set array[-0x7fffffff- 0x7fffffff]", 20, setArraySubII(arr, -0x7fffffff, 0x7fffffff, 20));

        array[index++] = Assert.expectEq( "set vector[1+2]", 10, setVectorAddII(vec, 1, 2, 10));
        array[index++] = Assert.expectEq( "set vector[1+ -2]", "Error #1125", setVectorAddII(vec, 1, -2, 11));
        array[index++] = Assert.expectEq( "set vector[0x7fffffff+0x7fffffff]", "Error #1125", setVectorAddII(vec, 0x7fffffff, 0x7fffffff, 12));
        array[index++] = Assert.expectEq( "set vector[0x7fffffff+ -0x7fffffff]", 13, setVectorAddII(vec, 0x7fffffff, -0x7fffffff, 13));
        array[index++] = Assert.expectEq( "set vector[-0x7fffffff+ -0x7fffffff]", "Error #1125", setVectorAddII(vec, -0x7fffffff, -0x7fffffff, 14));
        array[index++] = Assert.expectEq( "set vector[2+1], where 1 is const", 15, setVectorAddIC(vec, 2, 15));
        array[index++] = Assert.expectEq( "set vector[2-1], where 1 is const", 16, setVectorSubIC(vec, 2, 16));
        array[index++] = Assert.expectEq( "set vector[1+2], where 1 is const", 17, setVectorAddCI(vec, 2, 17));
        array[index++] = Assert.expectEq( "set vector[1-2], where 1 is const", "Error #1125", setVectorSubCI(vec, 2, 18));
        array[index++] = Assert.expectEq( "set vector[1-1], where 1 is const", 18, setVectorSubCI(vec, 1, 18));
        array[index++] = Assert.expectEq( "set vector[2-1]", 19, setVectorSubII(vec, 2, 1, 19));
        array[index++] = Assert.expectEq( "set vector[-0x7fffffff- 0x7fffffff]", "Error #1125", setVectorSubII(vec, -0x7fffffff, 0x7fffffff, 19));

        return (array);
}
