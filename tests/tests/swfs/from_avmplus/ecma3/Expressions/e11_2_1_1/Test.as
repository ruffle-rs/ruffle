/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11_2_1_1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Property Accessors";
    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var PROPERTY = new Array();


    array[item++] = Assert.expectEq(   "typeof this.NaN", "undefined", typeof (this.NaN) );
    array[item++] = Assert.expectEq(   "typeof this['NaN']", "undefined", typeof this['NaN'] );
    array[item++] = Assert.expectEq(   "typeof this.Infinity",     "undefined", typeof this.Infinity );
    array[item++] = Assert.expectEq(   "typeof this['Infinity']",  "undefined", typeof this['Infinity'] );
    array[item++] = Assert.expectEq(   "typeof this.parseInt",     "undefined", typeof this.parseInt );
    array[item++] = Assert.expectEq(   "typeof this['parseInt']",  "undefined", typeof this['parseInt']);
    array[item++] = Assert.expectEq(   "typeof this.parseFloat",   "undefined",typeof this.parseFloat );
    array[item++] = Assert.expectEq(   "typeof this['parseFloat']","undefined",typeof this['parseFloat']);
    array[item++] = Assert.expectEq(   "typeof this.escape",       "undefined",typeof this.escape );
    array[item++] = Assert.expectEq(   "typeof this['escape']",    "undefined",typeof this['escape']);
    array[item++] = Assert.expectEq(   "typeof this.unescape",     "undefined",typeof this.unescape );
    array[item++] = Assert.expectEq(   "typeof this['unescape']",  "undefined",typeof this['unescape']);
    array[item++] = Assert.expectEq(   "typeof this.isNaN",        "undefined",typeof this.isNaN );
    array[item++] = Assert.expectEq(   "typeof this['isNaN']",     "undefined",typeof this['isNaN']);
    array[item++] = Assert.expectEq(   "typeof this.isFinite",     "undefined",typeof this.isFinite );
    array[item++] = Assert.expectEq(   "typeof this['isFinite']",  "undefined",typeof this['isFinite']);
    array[item++] = Assert.expectEq(   "typeof this.Object",       "undefined", typeof this.Object);
    array[item++] = Assert.expectEq(   "typeof this['Object']",    "undefined", typeof this['Object']);
    array[item++] = Assert.expectEq(   "typeof this.Number",       "undefined", typeof this.Number);
    array[item++] = Assert.expectEq(   "typeof this['Number']",    "undefined", typeof this['Number']);
    array[item++] = Assert.expectEq(   "typeof this.Function",     "undefined", typeof this.Function);
    array[item++] = Assert.expectEq(   "typeof this['Function']",  "undefined", typeof this['Function']);
    array[item++] = Assert.expectEq(   "typeof this.Array",        "undefined", typeof this.Array);
    array[item++] = Assert.expectEq(   "typeof this['Array']",     "undefined", typeof this['Array']);
    array[item++] = Assert.expectEq(   "typeof this.String",       "undefined", typeof this.String);
    array[item++] = Assert.expectEq(   "typeof this['String']",    "undefined", typeof this['String']);
    array[item++] = Assert.expectEq(   "typeof this.Boolean",      "undefined", typeof this.Boolean);
    array[item++] = Assert.expectEq(   "typeof this['Boolean']",   "undefined", typeof this['Boolean']);
    array[item++] = Assert.expectEq(   "typeof this.Date",         "undefined", typeof this.Date);
    array[item++] = Assert.expectEq(   "typeof this['Date']",      "undefined", typeof this['Date']);
    array[item++] = Assert.expectEq(   "typeof this.Math",         "undefined", typeof this.Math);
    array[item++] = Assert.expectEq(   "typeof this['Math']",      "undefined", typeof this['Math']);

    // properties and  methods of Object objects

    array[item++] = Assert.expectEq(   "typeof Object.prototype", "object", typeof Object.prototype );
    array[item++] = Assert.expectEq(   "typeof Object['prototype']",    "object",typeof Object['prototype']);
    array[item++] = Assert.expectEq(   "typeof Object.toString",     "function", typeof Object.toString);
    array[item++] = Assert.expectEq(   "typeof Object['toString']",     "function", typeof Object['toString']);
    array[item++] = Assert.expectEq(   "typeof Object.valueOf",      "function", typeof Object.valueOf);
    array[item++] = Assert.expectEq(   "typeof Object['valueOf']",      "function", typeof Object['valueOf']);
    array[item++] = Assert.expectEq(   "typeof Object.constructor",  "object",typeof Object.constructor );
    array[item++] = Assert.expectEq(   "typeof Object['constructor']",  "object",typeof Object['constructor']);


    // properties of the Function object

    array[item++] = Assert.expectEq(   "typeof Function.prototype",              "function",typeof Function.prototype );
    array[item++] = Assert.expectEq(   "typeof Function['prototype']",        "function",typeof Function['prototype'] );
    array[item++] = Assert.expectEq(   "typeof Function.prototype.toString",     "function",typeof Function.prototype.toString );
    array[item++] = Assert.expectEq(   "typeof Function.prototype['toString']",     "function",typeof Function.prototype['toString'] );
    array[item++] = Assert.expectEq(   "typeof Function.prototype.length",       "number", typeof Function.prototype.length);
    array[item++] = Assert.expectEq(   "typeof Function.prototype['length']",       "number", typeof Function.prototype['length']);
    array[item++] = Assert.expectEq(   "typeof Function.prototype.valueOf",      "function",typeof Function.prototype.valueOf );
    array[item++] = Assert.expectEq(   "typeof Function.prototype['valueOf']",      "function",typeof Function.prototype['valueOf'] );

    //created extra property.  need to delete this at the end
    Function.prototype.myProperty = "hi";

    array[item++] = Assert.expectEq(   "typeof Function.prototype.myProperty",   "string",typeof Function.prototype.myProperty );
    array[item++] = Assert.expectEq(   "typeof Function.prototype['myProperty']",   "string",typeof Function.prototype['myProperty']);

    // properties of the Array object
    array[item++] = Assert.expectEq(   "typeof Array.prototype",    "object",typeof Array.prototype );
    array[item++] = Assert.expectEq(   "typeof Array['prototype']",    "object",typeof Array['prototype']);
    array[item++] = Assert.expectEq(   "typeof Array.length",       "number",typeof Array.length );
    array[item++] = Assert.expectEq(   "typeof Array['length']",       "number",typeof Array['length']);
    array[item++] = Assert.expectEq(   "typeof Array.prototype.constructor",  "object",typeof Array.prototype.constructor );
    array[item++] = Assert.expectEq(   "typeof Array.prototype['constructor']",  "object",typeof Array.prototype['constructor']);
    array[item++] = Assert.expectEq(   "typeof Array.prototype.toString",     "function",typeof Array.prototype.toString );
    array[item++] = Assert.expectEq(   "typeof Array.prototype['toString']",     "function",typeof Array.prototype['toString']);
    array[item++] = Assert.expectEq(   "typeof Array.prototype.join",         "function",typeof Array.prototype.join );
    array[item++] = Assert.expectEq(   "typeof Array.prototype['join']",         "function",typeof Array.prototype['join']);
    array[item++] = Assert.expectEq(   "typeof Array.prototype.reverse",      "function",typeof Array.prototype.reverse );
    array[item++] = Assert.expectEq(   "typeof Array.prototype['reverse']",      "function",typeof Array.prototype['reverse']);
    array[item++] = Assert.expectEq(   "typeof Array.prototype.sort",         "function",typeof Array.prototype.sort );
    array[item++] = Assert.expectEq(   "typeof Array.prototype['sort']",         "function",typeof Array.prototype['sort']);


    // properties of the String object
    array[item++] = Assert.expectEq(   "typeof String.prototype",    "object",typeof String.prototype );
    array[item++] = Assert.expectEq(   "typeof String['prototype']",    "object",typeof String['prototype'] );
    array[item++] = Assert.expectEq(   "typeof String.fromCharCode", "function",typeof String.fromCharCode );
    array[item++] = Assert.expectEq(   "typeof String['fromCharCode']", "function",typeof String['fromCharCode'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.toString",     "function",typeof String.prototype.toString );
    array[item++] = Assert.expectEq(   "typeof String.prototype['toString']",     "function",typeof String.prototype['toString'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.constructor",  "object",typeof String.prototype.constructor );
    array[item++] = Assert.expectEq(   "typeof String.prototype['constructor']",  "object",typeof String.prototype['constructor'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.valueOf",      "function",typeof String.prototype.valueOf );
    array[item++] = Assert.expectEq(   "typeof String.prototype['valueOf']",      "function",typeof String.prototype['valueOf'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.charAt",       "function", typeof String.prototype.charAt);
    array[item++] = Assert.expectEq(   "typeof String.prototype['charAt']",       "function", typeof String.prototype['charAt']);
    array[item++] = Assert.expectEq(   "typeof String.prototype.charCodeAt",   "function", typeof String.prototype.charCodeAt);
    array[item++] = Assert.expectEq(   "typeof String.prototype['charCodeAt']",   "function", typeof String.prototype['charCodeAt']);
    array[item++] = Assert.expectEq(   "typeof String.prototype.indexOf",      "function",typeof String.prototype.indexOf );
    array[item++] = Assert.expectEq(   "typeof String.prototype['indexOf']",      "function",typeof String.prototype['indexOf'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.lastIndexOf",  "function",typeof String.prototype.lastIndexOf );
    array[item++] = Assert.expectEq(   "typeof String.prototype['lastIndexOf']",  "function",typeof String.prototype['lastIndexOf'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.split",        "function", typeof String.prototype.split);
    array[item++] = Assert.expectEq(   "typeof String.prototype['split']",        "function", typeof String.prototype['split']);
    array[item++] = Assert.expectEq(   "typeof String.prototype.substring",    "function",typeof String.prototype.substring );
    array[item++] = Assert.expectEq(   "typeof String.prototype['substring']",    "function",typeof String.prototype['substring'] );
    array[item++] = Assert.expectEq(   "typeof String.prototype.toLowerCase",  "function", typeof String.prototype.toLowerCase);
    array[item++] = Assert.expectEq(   "typeof String.prototype['toLowerCase']",  "function", typeof String.prototype['toLowerCase']);
    array[item++] = Assert.expectEq(   "typeof String.prototype.toUpperCase",  "function", typeof String.prototype.toUpperCase);
    array[item++] = Assert.expectEq(   "typeof String.prototype['toUpperCase']",  "function", typeof String.prototype['toUpperCase']);
    array[item++] = Assert.expectEq(   "typeof String.prototype.length",       "undefined",typeof String.prototype.length );
    array[item++] = Assert.expectEq(   "typeof String.prototype['length']",       "undefined",typeof String.prototype['length'] );


    // properties of the Boolean object
    array[item++] = Assert.expectEq(   "typeof Boolean.prototype",    "object",typeof Boolean.prototype );
    array[item++] = Assert.expectEq(   "typeof Boolean['prototype']",    "object",typeof Boolean['prototype'] );
    array[item++] = Assert.expectEq(   "typeof Boolean.constructor",  "object",typeof Boolean.constructor );
    array[item++] = Assert.expectEq(   "typeof Boolean['constructor']",  "object",typeof Boolean['constructor'] );
    array[item++] = Assert.expectEq(   "typeof Boolean.prototype.valueOf",      "function",typeof Boolean.prototype.valueOf );
    array[item++] = Assert.expectEq(   "typeof Boolean.prototype['valueOf']",      "function",typeof Boolean.prototype['valueOf']);
    array[item++] = Assert.expectEq(   "typeof Boolean.prototype.toString",     "function", typeof Boolean.prototype.toString);
    array[item++] = Assert.expectEq(   "typeof Boolean.prototype['toString']",     "function", typeof Boolean.prototype['toString']);

    // properties of the Number object

    array[item++] = Assert.expectEq(   "typeof Number.MAX_VALUE",    "number",typeof Number.MAX_VALUE );
    array[item++] = Assert.expectEq(   "typeof Number['MAX_VALUE']",    "number",typeof Number['MAX_VALUE']);
    array[item++] = Assert.expectEq(   "typeof Number.MIN_VALUE",    "number", typeof Number.MIN_VALUE);
    array[item++] = Assert.expectEq(   "typeof Number['MIN_VALUE']",    "number", typeof Number['MIN_VALUE']);
    array[item++] = Assert.expectEq(   "typeof Number.NaN",          "number", typeof Number.NaN);
    array[item++] = Assert.expectEq(   "typeof Number['NaN']",          "number", typeof Number['NaN']);
    array[item++] = Assert.expectEq(   "typeof Number.NEGATIVE_INFINITY",    "number", typeof Number.NEGATIVE_INFINITY);
    array[item++] = Assert.expectEq(   "typeof Number['NEGATIVE_INFINITY']",    "number", typeof Number['NEGATIVE_INFINITY']);
    array[item++] = Assert.expectEq(   "typeof Number.POSITIVE_INFINITY",    "number", typeof Number.POSITIVE_INFINITY);
    array[item++] = Assert.expectEq(   "typeof Number['POSITIVE_INFINITY']",    "number", typeof Number['POSITIVE_INFINITY']);
    array[item++] = Assert.expectEq(   "typeof Number.prototype.toString",     "function",typeof Number.prototype.toString );
    array[item++] = Assert.expectEq(   "typeof Number.prototype['toString']",     "function",typeof Number.prototype['toString'] );
    array[item++] = Assert.expectEq(   "typeof Number.prototype.constructor",  "object", typeof Number.prototype.constructor);
    array[item++] = Assert.expectEq(   "typeof Number.prototype['constructor']",  "object", typeof Number.prototype['constructor']);
    array[item++] = Assert.expectEq(   "typeof Number.prototype.valueOf",        "function",typeof Number.prototype.valueOf );
        array[item++] = Assert.expectEq(   "typeof Number.prototype['valueOf']",        "function",typeof Number.prototype['valueOf'] );

    // properties of the Math Object.
    array[item++] = Assert.expectEq(   "typeof Math.E",        "number", typeof Math.E);
    array[item++] = Assert.expectEq(   "typeof Math['E']",        "number", typeof Math['E']);
    array[item++] = Assert.expectEq(   "typeof Math.LN10",     "number", typeof Math.LN10);
        array[item++] = Assert.expectEq(   "typeof Math['LN10']",     "number", typeof Math['LN10']);
    array[item++] = Assert.expectEq(   "typeof Math.LN2",      "number", typeof Math.LN2);
        array[item++] = Assert.expectEq(   "typeof Math['LN2']",      "number", typeof Math['LN2']);
    array[item++] = Assert.expectEq(   "typeof Math.LOG2E",    "number", typeof Math.LOG2E);
        array[item++] = Assert.expectEq(   "typeof Math['LOG2E']",    "number", typeof Math['LOG2E']);
    array[item++] = Assert.expectEq(   "typeof Math.LOG10E",   "number", typeof Math.LOG10E);
        array[item++] = Assert.expectEq(   "typeof Math['LOG10E']",   "number", typeof Math['LOG10E']);
    array[item++] = Assert.expectEq(   "typeof Math.PI",       "number", typeof Math.PI);
        array[item++] = Assert.expectEq(   "typeof Math['PI']",       "number", typeof Math['PI']);
    array[item++] = Assert.expectEq(   "typeof Math.SQRT1_2",  "number", typeof Math.SQRT1_2);
        array[item++] = Assert.expectEq(   "typeof Math['SQRT1_2']",  "number", typeof Math['SQRT1_2']);
    array[item++] = Assert.expectEq(   "typeof Math.SQRT2",    "number", typeof Math.SQRT2);
        array[item++] = Assert.expectEq(   "typeof Math['SQRT2']",    "number", typeof Math['SQRT2']);
    array[item++] = Assert.expectEq(   "typeof Math.abs",      "function", typeof Math.abs);
        array[item++] = Assert.expectEq(   "typeof Math['abs']",      "function", typeof Math['abs']);
    array[item++] = Assert.expectEq(   "typeof Math.acos",     "function", typeof Math.acos);
        array[item++] = Assert.expectEq(   "typeof Math['acos']",     "function", typeof Math['acos']);
    array[item++] = Assert.expectEq(   "typeof Math.asin",     "function", typeof Math.asin);
        array[item++] = Assert.expectEq(   "typeof Math['asin']",     "function", typeof Math['asin']);
    array[item++] = Assert.expectEq(   "typeof Math.atan",     "function", typeof Math.atan);
        array[item++] = Assert.expectEq(   "typeof Math['atan']",     "function", typeof Math['atan']);
    array[item++] = Assert.expectEq(   "typeof Math.atan2",    "function", typeof Math.atan2);
        array[item++] = Assert.expectEq(   "typeof Math['atan2']",    "function", typeof Math['atan2']);
    array[item++] = Assert.expectEq(   "typeof Math.ceil",     "function", typeof Math.ceil);
        array[item++] = Assert.expectEq(   "typeof Math['ceil']",     "function", typeof Math['ceil']);
    array[item++] = Assert.expectEq(   "typeof Math.cos",      "function", typeof Math.cos);
        array[item++] = Assert.expectEq(   "typeof Math['cos']",      "function", typeof Math['cos']);
    array[item++] = Assert.expectEq(   "typeof Math.exp",      "function", typeof Math.exp);
        array[item++] = Assert.expectEq(   "typeof Math['exp']",      "function", typeof Math['exp']);
    array[item++] = Assert.expectEq(   "typeof Math.floor",    "function", typeof Math.floor);
        array[item++] = Assert.expectEq(   "typeof Math['floor']",    "function", typeof Math['floor']);
    array[item++] = Assert.expectEq(   "typeof Math.log",      "function", typeof Math.log);
        array[item++] = Assert.expectEq(   "typeof Math['log']",      "function", typeof Math['log']);
    array[item++] = Assert.expectEq(   "typeof Math.max",      "function", typeof Math.max);
        array[item++] = Assert.expectEq(   "typeof Math['max']",      "function", typeof Math['max']);
    array[item++] = Assert.expectEq(   "typeof Math.min",      "function", typeof Math.min);
        array[item++] = Assert.expectEq(   "typeof Math['min']",      "function", typeof Math['min']);
    array[item++] = Assert.expectEq(   "typeof Math.pow",      "function", typeof Math.pow);
        array[item++] = Assert.expectEq(   "typeof Math['pow']",      "function", typeof Math['pow']);
    array[item++] = Assert.expectEq(   "typeof Math.random",   "function", typeof Math.random);
        array[item++] = Assert.expectEq(   "typeof Math['random']",   "function", typeof Math['random']);
    array[item++] = Assert.expectEq(   "typeof Math.round",    "function", typeof Math.round);
        array[item++] = Assert.expectEq(   "typeof Math['round']",    "function", typeof Math['round']);
    array[item++] = Assert.expectEq(   "typeof Math.sin",      "function", typeof Math.sin);
        array[item++] = Assert.expectEq(   "typeof Math['sin']",      "function", typeof Math['sin']);
    array[item++] = Assert.expectEq(   "typeof Math.sqrt",     "function", typeof Math.sqrt);
        array[item++] = Assert.expectEq(   "typeof Math['sqrt']",     "function", typeof Math['sqrt']);
    array[item++] = Assert.expectEq(   "typeof Math.tan",      "function", typeof Math.tan);
       array[item++] = Assert.expectEq(   "typeof Math['tan']",      "function", typeof Math['tan']);

    // properties of the Date object
    array[item++] = Assert.expectEq(   "typeof Date.parse",        "function", typeof Date.parse);
    array[item++] = Assert.expectEq(   "typeof Date['parse']",        "function", typeof Date['parse']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype",    "object", typeof Date.prototype);
    array[item++] = Assert.expectEq(   "typeof Date['prototype']",    "object", typeof Date['prototype']);
    array[item++] = Assert.expectEq(   "typeof Date.UTC",          "function", typeof Date.UTC);
    array[item++] = Assert.expectEq(   "typeof Date['UTC']",          "function", typeof Date['UTC']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.constructor",    "object", typeof Date.prototype.constructor);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['constructor']",    "object", typeof Date.prototype['constructor']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.toString",       "function", typeof Date.prototype.toString);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['toString']",       "function", typeof Date.prototype['toString']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.valueOf",        "function", typeof Date.prototype.valueOf);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['valueOf']",        "function", typeof Date.prototype['valueOf']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getTime",        "function", typeof Date.prototype.getTime);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getTime']",        "function", typeof Date.prototype['getTime']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getYear",        "undefined", typeof Date.prototype.getYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getYear']",        "undefined", typeof Date.prototype['getYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getFullYear",    "function", typeof Date.prototype.getFullYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getFullYear']",    "function", typeof Date.prototype['getFullYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCFullYear", "function", typeof Date.prototype.getUTCFullYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCFullYear']", "function", typeof Date.prototype['getUTCFullYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getMonth",       "function", typeof Date.prototype.getMonth);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getMonth']",       "function", typeof Date.prototype['getMonth']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCMonth",    "function", typeof Date.prototype.getUTCMonth);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCMonth']",    "function", typeof Date.prototype['getUTCMonth']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getDate",        "function", typeof Date.prototype.getDate);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getDate']",        "function", typeof Date.prototype['getDate']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCDate",     "function", typeof Date.prototype.getUTCDate);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCDate']",     "function", typeof Date.prototype['getUTCDate']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getDay",         "function", typeof Date.prototype.getDay);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getDay']",         "function", typeof Date.prototype['getDay']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCDay",      "function", typeof Date.prototype.getUTCDay);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCDay']",      "function", typeof Date.prototype['getUTCDay']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getHours",       "function", typeof Date.prototype.getHours);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getHours']",       "function", typeof Date.prototype['getHours']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCHours",    "function", typeof Date.prototype.getUTCHours);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCHours']",    "function", typeof Date.prototype['getUTCHours']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getMinutes",     "function", typeof Date.prototype.getMinutes);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getMinutes']",     "function", typeof Date.prototype['getMinutes']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCMinutes",  "function", typeof Date.prototype.getUTCMinutes);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCMinutes']",  "function", typeof Date.prototype['getUTCMinutes']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getSeconds",     "function", typeof Date.prototype.getSeconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getSeconds']",     "function", typeof Date.prototype['getSeconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCSeconds",  "function", typeof Date.prototype.getUTCSeconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCSeconds']",  "function", typeof Date.prototype['getUTCSeconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getMilliseconds","function", typeof Date.prototype.getMilliseconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getMilliseconds']","function", typeof Date.prototype['getMilliseconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.getUTCMilliseconds", "function", typeof Date.prototype.getUTCMilliseconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['getUTCMilliseconds']", "function", typeof Date.prototype['getUTCMilliseconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setTime",        "function", typeof Date.prototype.setTime);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setTime']",        "function", typeof Date.prototype['setTime']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setMilliseconds","function", typeof Date.prototype.setMilliseconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setMilliseconds']","function", typeof Date.prototype['setMilliseconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCMilliseconds", "function", typeof Date.prototype.setUTCMilliseconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCMilliseconds']", "function", typeof Date.prototype['setUTCMilliseconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setSeconds",     "function", typeof Date.prototype.setSeconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setSeconds']",     "function", typeof Date.prototype['setSeconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCSeconds",  "function", typeof Date.prototype.setUTCSeconds);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCSeconds']",  "function", typeof Date.prototype['setUTCSeconds']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setMinutes",     "function", typeof Date.prototype.setMinutes);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setMinutes']",     "function", typeof Date.prototype['setMinutes']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCMinutes",  "function", typeof Date.prototype.setUTCMinutes);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCMinutes']",  "function", typeof Date.prototype['setUTCMinutes']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setHours",       "function", typeof Date.prototype.setHours);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setHours']",       "function", typeof Date.prototype['setHours']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCHours",    "function", typeof Date.prototype.setUTCHours);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCHours']",    "function", typeof Date.prototype['setUTCHours']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setDate",        "function", typeof Date.prototype.setDate);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setDate']",        "function", typeof Date.prototype['setDate']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCDate",     "function", typeof Date.prototype.setUTCDate);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCDate']",     "function", typeof Date.prototype['setUTCDate']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setMonth",       "function", typeof Date.prototype.setMonth);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setMonth']",       "function", typeof Date.prototype['setMonth']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCMonth",    "function", typeof Date.prototype.setUTCMonth);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCMonth']",    "function", typeof Date.prototype['setUTCMonth']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setFullYear",    "function", typeof Date.prototype.setFullYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setFullYear']",    "function", typeof Date.prototype['setFullYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setUTCFullYear", "function", typeof Date.prototype.setUTCFullYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setUTCFullYear']", "function", typeof Date.prototype['setUTCFullYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.setYear",        "undefined", typeof Date.prototype.setYear);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['setYear']",        "undefined", typeof Date.prototype['setYear']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.toLocaleString", "function", typeof Date.prototype.toLocaleString);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['toLocaleString']", "function", typeof Date.prototype['toLocaleString']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.toUTCString",    "function", typeof Date.prototype.toUTCString);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['toUTCString']",    "function", typeof Date.prototype['toUTCString']);
    array[item++] = Assert.expectEq(   "typeof Date.prototype.toGMTString",    "undefined", typeof Date.prototype.toGMTString);
    array[item++] = Assert.expectEq(   "typeof Date.prototype['toGMTString']",    "undefined", typeof Date.prototype['toGMTString']);

    for ( var i = 0, RESULT; i < PROPERTY.length; i++ ) {
       RESULT = typeof (PROPERTY[i].object.PROPERTY[i].name);


        array[item++] = Assert.expectEq( 
                                        "typeof " + PROPERTY[i].object + "." + PROPERTY[i].name,
                                        PROPERTY[i].type+"",
                                        RESULT );


    RESULT = typeof (PROPERTY[i].object['PROPERTY[i].name']);


        array[item++] = Assert.expectEq( 
                                        "typeof " + PROPERTY[i].object + "['" + PROPERTY[i].name +"']",
                                        PROPERTY[i].type,
                                        RESULT );

    }
    
    //restore.  deleted the extra property created so it doesn't interfere with another testcase
    //in ATS.
    delete Function.prototype.myProperty;
    return array;
}

function MyObject( arg0, arg1, arg2, arg3, arg4 ) {
    this.name   = arg0;
}
function Property( object, name, type ) {
    this.object = object;
    this.name = name;
    this.type = type;
}
