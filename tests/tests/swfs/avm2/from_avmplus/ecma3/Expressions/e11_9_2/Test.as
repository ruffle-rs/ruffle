/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11.9.2";
//     var VERSION = "ECMA_1";
    var BUGNUMBER="77391";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // type x and type y are the same.  if type x is undefined or null, return true

    array[item++] = Assert.expectEq(     "void 0 == void 0",        false,   void 0 != void 0 );
    array[item++] = Assert.expectEq(     "null == null",           false,   null != null );

    //  if x is NaN, return false. if y is NaN, return false.

    array[item++] = Assert.expectEq(     "NaN != NaN",             true,  Number.NaN != Number.NaN );
    array[item++] = Assert.expectEq(     "NaN != 0",               true,  Number.NaN != 0 );
    array[item++] = Assert.expectEq(     "0 != NaN",               true,  0 != Number.NaN );
    array[item++] = Assert.expectEq(     "NaN != Infinity",        true,  Number.NaN != Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Infinity != NaN",        true,  Number.POSITIVE_INFINITY != Number.NaN );

    // if x is the same number value as y, return true.

    array[item++] = Assert.expectEq(     "Number.MAX_VALUE != Number.MAX_VALUE",   false,   Number.MAX_VALUE != Number.MAX_VALUE );
    array[item++] = Assert.expectEq(     "Number.MIN_VALUE != Number.MIN_VALUE",   false,   Number.MIN_VALUE != Number.MIN_VALUE );
    array[item++] = Assert.expectEq(     "Number.POSITIVE_INFINITY != Number.POSITIVE_INFINITY",   false,   Number.POSITIVE_INFINITY != Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(     "Number.NEGATIVE_INFINITY != Number.NEGATIVE_INFINITY",   false,   Number.NEGATIVE_INFINITY != Number.NEGATIVE_INFINITY );

    //  if xis 0 and y is -0, return true.   if x is -0 and y is 0, return true.

    array[item++] = Assert.expectEq(     "0 != 0",                 false,   0 != 0 );
    array[item++] = Assert.expectEq(     "0 != -0",                false,   0 != -0 );
    array[item++] = Assert.expectEq(     "-0 != 0",                false,   -0 != 0 );
    array[item++] = Assert.expectEq(     "-0 != -0",               false,   -0 != -0 );

    // return false.

    array[item++] = Assert.expectEq(     "0.9 != 1",               true,  0.9 != 1 );
    array[item++] = Assert.expectEq(     "0.999999 != 1",          true,  0.999999 != 1 );
    array[item++] = Assert.expectEq(     "0.9999999999 != 1",      true,  0.9999999999 != 1 );
    array[item++] = Assert.expectEq(     "0.9999999999999 != 1",   true,  0.9999999999999 != 1 );

    // type x and type y are the same type, but not numbers.


    // x and y are strings.  return true if x and y are exactly the same sequence of characters.
    // otherwise, return false.

    array[item++] = Assert.expectEq(     "'hello' != 'hello'",         false,   "hello" != "hello" );

    // x and y are booleans.  return true if both are true or both are false.

    array[item++] = Assert.expectEq(     "true != true",               false,   true != true );
    array[item++] = Assert.expectEq(     "false != false",             false,   false != false );
    array[item++] = Assert.expectEq(     "true != false",              true,  true != false );
    array[item++] = Assert.expectEq(     "false != true",              true,  false != true );

    // return true if x and y refer to the same object.  otherwise return false.

    array[item++] = Assert.expectEq(     "new MyObject(true) != new MyObject(true)",   true,  new MyObject(true) != new MyObject(true) );
    array[item++] = Assert.expectEq(     "new Boolean(true) != new Boolean(true)",     false,  new Boolean(true) != new Boolean(true) );
    array[item++] = Assert.expectEq(     "new Boolean(false) != new Boolean(false)",   false,  new Boolean(false) != new Boolean(false) );

    x = new MyObject(true); y = x; z = x;
    array[item++] = Assert.expectEq(     "x = new MyObject(true); y = x; z = x; z != y",   false,   z != y );
    x = new MyObject(false); y = x; z = x;
    array[item++] = Assert.expectEq(     "x = new MyObject(false); y = x; z = x; z != y",  false,   z != y );
    x = new Boolean(true); y = x; z = x;
    array[item++] = Assert.expectEq(     "x = new Boolean(true); y = x; z = x; z != y",   false,  z != y);
    x = new Boolean(false); y = x; z = x;
    array[item++] = Assert.expectEq(     "x = new Boolean(false); y = x; z = x; z != y",   false,  z != y );

    array[item++] = Assert.expectEq(     "new Boolean(true) != new Boolean(true)",     false,  new Boolean(true) != new Boolean(true) );
    array[item++] = Assert.expectEq(     "new Boolean(false) != new Boolean(false)",   false,  new Boolean(false) != new Boolean(false) );

    // if x is null and y is undefined, return true.  if x is undefined and y is null return true.

    array[item++] = Assert.expectEq(     "null != void 0",             false,   null != void 0 );
    array[item++] = Assert.expectEq(     "void 0 != null",             false,   void 0 != null );

    // if type(x) is Number and type(y) is string, return the result of the comparison x != ToNumber(y).

    array[item++] = Assert.expectEq(     "1 != '1'",                   false,   1 != '1' );
    array[item++] = Assert.expectEq(     "255 != '0xff'",               false,  255 != '0xff' );
    array[item++] = Assert.expectEq(     "0 != '\\r'",                  false,   0 != "\r" );
    array[item++] = Assert.expectEq(     "1e19 != '1e19'",             false,   1e19 != "1e19" );


    array[item++] = Assert.expectEq(     "new Boolean(true) != true",  false,   true != new Boolean(true) );
    array[item++] = Assert.expectEq(     "new MyObject(true) != true", false,   true != new MyObject(true) );

    array[item++] = Assert.expectEq(     "new Boolean(false) != false",    false,   new Boolean(false) != false );
    array[item++] = Assert.expectEq(     "new MyObject(false) != false",   false,   new MyObject(false) != false );

    array[item++] = Assert.expectEq(     "true != new Boolean(true)",      false,   true != new Boolean(true) );
    array[item++] = Assert.expectEq(     "true != new MyObject(true)",     false,   true != new MyObject(true) );

    array[item++] = Assert.expectEq(     "false != new Boolean(false)",    false,   false != new Boolean(false) );
    array[item++] = Assert.expectEq(     "false != new MyObject(false)",   false,   false != new MyObject(false) );

    return ( array );
}

function MyObject( value ) {
    this.value = value;
    //this.valueOf = new Function( "return this.value" );
    this.valueOf = function(){return this.value};
}
