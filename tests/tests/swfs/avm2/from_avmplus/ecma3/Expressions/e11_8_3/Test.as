/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_8_3";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "true <= false",              false,      true <= false );
    array[item++] = Assert.expectEq(  "false <= true",              true,       false <= true );
    array[item++] = Assert.expectEq(  "false <= false",             true,      false <= false );
    array[item++] = Assert.expectEq(  "true <= true",               true,      true <= true );

    array[item++] = Assert.expectEq(  "new Boolean(true) <= new Boolean(true)",     true,  new Boolean(true) <= new Boolean(true) );
    array[item++] = Assert.expectEq(  "new Boolean(true) <= new Boolean(false)",    false,  new Boolean(true) <= new Boolean(false) );
    array[item++] = Assert.expectEq(  "new Boolean(false) <= new Boolean(true)",    true,   new Boolean(false) <= new Boolean(true) );
    array[item++] = Assert.expectEq(  "new Boolean(false) <= new Boolean(false)",   true,  new Boolean(false) <= new Boolean(false) );

    array[item++] = Assert.expectEq(  "new MyObject(Infinity) <= new MyObject(Infinity)",   true,  new MyObject( Number.POSITIVE_INFINITY ) <= new MyObject( Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "new MyObject(-Infinity) <= new MyObject(Infinity)",  true,   new MyObject( Number.NEGATIVE_INFINITY ) <= new MyObject( Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "new MyObject(-Infinity) <= new MyObject(-Infinity)", true,  new MyObject( Number.NEGATIVE_INFINITY ) <= new MyObject( Number.NEGATIVE_INFINITY) );

    array[item++] = Assert.expectEq(  "new MyValueObject(false) <= new MyValueObject(true)",  true,   new MyValueObject(false) <= new MyValueObject(true) );
    array[item++] = Assert.expectEq(  "new MyValueObject(true) <= new MyValueObject(true)",   true,  new MyValueObject(true) <= new MyValueObject(true) );
    array[item++] = Assert.expectEq(  "new MyValueObject(false) <= new MyValueObject(false)", true,  new MyValueObject(false) <= new MyValueObject(false) );

    array[item++] = Assert.expectEq(  "new MyStringObject(false) <= new MyStringObject(true)",  true,   new MyStringObject(false) <= new MyStringObject(true) );
    array[item++] = Assert.expectEq(  "new MyStringObject(true) <= new MyStringObject(true)",   true,  new MyStringObject(true) <= new MyStringObject(true) );
    array[item++] = Assert.expectEq(  "new MyStringObject(false) <= new MyStringObject(false)", true,  new MyStringObject(false) <= new MyStringObject(false) );

    array[item++] = Assert.expectEq(  "Number.NaN <= Number.NaN",   false,     Number.NaN <= Number.NaN );
    array[item++] = Assert.expectEq(  "0 <= Number.NaN",            false,     0 <= Number.NaN );
    array[item++] = Assert.expectEq(  "Number.NaN <= 0",            false,     Number.NaN <= 0 );

    array[item++] = Assert.expectEq(  "0 <= -0",                    true,      0 <= -0 );
    array[item++] = Assert.expectEq(  "-0 <= 0",                    true,      -0 <= 0 );

    array[item++] = Assert.expectEq(  "Infinity <= 0",                  false,      Number.POSITIVE_INFINITY <= 0 );
    array[item++] = Assert.expectEq(  "Infinity <= Number.MAX_VALUE",   false,      Number.POSITIVE_INFINITY <= Number.MAX_VALUE );
    array[item++] = Assert.expectEq(  "Infinity <= Infinity",           true,       Number.POSITIVE_INFINITY <= Number.POSITIVE_INFINITY );

    array[item++] = Assert.expectEq(  "0 <= Infinity",                  true,       0 <= Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(  "Number.MAX_VALUE <= Infinity",   true,       Number.MAX_VALUE <= Number.POSITIVE_INFINITY );

    array[item++] = Assert.expectEq(  "0 <= -Infinity",                 false,      0 <= Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(  "Number.MAX_VALUE <= -Infinity",  false,      Number.MAX_VALUE <= Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(  "-Infinity <= -Infinity",         true,       Number.NEGATIVE_INFINITY <= Number.NEGATIVE_INFINITY );

    array[item++] = Assert.expectEq(  "-Infinity <= 0",                 true,       Number.NEGATIVE_INFINITY <= 0 );
    array[item++] = Assert.expectEq(  "-Infinity <= -Number.MAX_VALUE", true,       Number.NEGATIVE_INFINITY <= -Number.MAX_VALUE );
    array[item++] = Assert.expectEq(  "-Infinity <= Number.MIN_VALUE",  true,       Number.NEGATIVE_INFINITY <= Number.MIN_VALUE );

    array[item++] = Assert.expectEq(  "'string' <= 'string'",           true,       'string' <= 'string' );
    array[item++] = Assert.expectEq(  "'astring' <= 'string'",          true,       'astring' <= 'string' );
    array[item++] = Assert.expectEq(  "'strings' <= 'stringy'",         true,       'strings' <= 'stringy' );
    array[item++] = Assert.expectEq(  "'strings' <= 'stringier'",       false,       'strings' <= 'stringier' );
    array[item++] = Assert.expectEq(  "'string' <= 'astring'",          false,      'string' <= 'astring' );
    array[item++] = Assert.expectEq(  "'string' <= 'strings'",          true,       'string' <= 'strings' );
    array[item++] = Assert.expectEq(  "'string' <= 'str'",              false,       'string' <= 'str' );
    
    array[item++] = Assert.expectEq(  "'string' <= undefined",          false,       'string' <= undefined );
    array[item++] = Assert.expectEq(  "undefined  <= 'string'",          false,       undefined  <= 'string' );
    array[item++] = Assert.expectEq(  "6.9 <= undefined",               false,       6.9 <= undefined );
    array[item++] = Assert.expectEq(  "undefined <= 343",               false,       undefined <= 343);

    array[item++] = Assert.expectEq(  "44 <= 55",          true,       44 <= 55 );
    array[item++] = Assert.expectEq(  "55 <= 44",          false,       55 <= 44 );
    array[item++] = Assert.expectEq(  "56.43 <= 65.0",          true,       56.43 <= 65.0 );
    array[item++] = Assert.expectEq(  "65.0 <= 56.43",          false,       65.0 <= 56.43 );
    array[item++] = Assert.expectEq(  "43247503.43 <= 945540654.654",          true,       43247503.43 <= 945540654.654 );
    array[item++] = Assert.expectEq(  "43247503.43<=945540654.654",          true,       43247503.43<=945540654.654 );
    array[item++] = Assert.expectEq(  "43247503.43<= 945540654.654",          true,       43247503.43<= 945540654.654 );
    array[item++] = Assert.expectEq(  "43247503.43   <=  945540654.654",          true,       43247503.43   <=  945540654.654 );
    array[item++] = Assert.expectEq(  "-56.43 <= 65.0",          true,       -56.43 <= 65.0 );
    array[item++] = Assert.expectEq(  "-56.43 <= -65.0",          false,       -56.43 <= -65.0 );
    array[item++] = Assert.expectEq(  "100 <= 100",               true,      100 <= 100 );
    array[item++] = Assert.expectEq(  "-56.43 <= -56.43",          true,       -56.43 <= -56.43 );

    return ( array );
}
function MyObject(value) {
    this.value = value;
    //this.valueOf = new Function( "return this.value" );
    //this.toString = new Function( "return this.value +''" );
    this.valueOf = function(){return this.value};
    this.toString = function(){return this.value+''};
}
function MyValueObject(value) {
    this.value = value;
//  this.valueOf = new Function( "return this.value" );
    this.valueOf = function(){return this.value};
}
function MyStringObject(value) {
    this.value = value;
//  this.toString = new Function( "return this.value +''" );
    this.toString = function(){return this.value+''};
}
