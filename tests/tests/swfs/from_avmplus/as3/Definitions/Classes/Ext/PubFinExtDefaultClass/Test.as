/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import DefaultClass.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";                // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";                 // Version of JavaScript or ECMA
// var TITLE   = "Public Final Class Extends Default Class";    // Provide ECMA section title or a description
var BUGNUMBER = "";



// *******************************************
//  access default method from
//  outside of class
// *******************************************

var DYNEXTDCLASS = new PubFinExtDefaultClass();
var arr = new Array(10, 15, 20, 25, 30);

Assert.expectEq( "*** Access Default Method from Default method of sub class  ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testSubGetSetArray(arr)", arr, (DYNEXTDCLASS.testSubGetSetArray(arr)) );


// ********************************************
// access default method from a public
// method of a sub class
//
// ********************************************
var arr = new Array(1, 5);
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "*** Access default method from public method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()", arr, (DYNEXTDCLASS.pubSubSetArray(arr), DYNEXTDCLASS.pubSubGetArray()) );


// ********************************************
// access default method from a private
// method of a sub class
//
// ********************************************
var arr = new Array(2, 4, 6);
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "*** Access default method from private method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testPrivSubArray(arr)", arr, DYNEXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access default method from a final
// method of a sub class
//
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' method from 'final' method of sub class", arr, (DYNEXTDCLASS.testFinSubArray(arr)) );

// ********************************************
// access default method from a static
// method of a sub class
//
// ********************************************
/*
var arr = new Array(1, 5);
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "*** Access default method from static method of sub class ***", 1, 1 );
Assert.expectEq( "DYNEXTDCLASS.testStatSubArray(arr)", arr, (DYNEXTDCLASS.testStatSubArray(arr)) );
*/

// ********************************************
// access default method from a public
// final method of a sub class
// ********************************************

arr = new Array( 1, 2, 3 );
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' method from 'public final' method of sub class", arr,
             (DYNEXTDCLASS.pubFinSubSetArray(arr), DYNEXTDCLASS.pubFinSubGetArray()) );

// ********************************************
// access default method from a final
// private method of a sub class
// ********************************************

arr = new Array( 4, 5 );
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' method from 'private final' method of sub class", arr, (DYNEXTDCLASS.testPrivFinSubArray(arr)) );

// ********************************************
// access default method from a private
// method of a sub class
// ********************************************

arr = new Array( 6, 7 );
DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' method from 'private' method of sub class", arr, DYNEXTDCLASS.testPrivSubArray(arr) );

// ********************************************
// access default method from a virtual
// method of a sub class
// ********************************************

Assert.expectEq( "access 'default' method from 'virtual' method of sub class", arr,
              DYNEXTDCLASS.testVirtSubArray(arr) );

// ********************************************
// access default method from a virtual
// public method of a sub class
// ********************************************

Assert.expectEq( "access 'default' method from 'public virtual' method of sub class", arr,
             (DYNEXTDCLASS.pubVirtSubSetArray(arr), DYNEXTDCLASS.pubVirtSubGetArray()) );

// ********************************************
// access default method from a virtual
// private method of a sub class
// ********************************************

Assert.expectEq( "access 'default' method from 'private virtual' method of sub class", arr,
              DYNEXTDCLASS.testPrivVirtSubArray(arr) );



/* Access properties of parent class */

// ********************************************
// access default property from
// default method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'default' method of sub class", arr,
                (DYNEXTDCLASS.testSubGetSetDPArray(arr)) );

// ********************************************
// access default property from
// final method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'final' method of sub class", arr,
                (DYNEXTDCLASS.testFinSubDPArray(arr)) );

// ********************************************
// access default property from
// virtual method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'virtual' method of sub class", arr,
                (DYNEXTDCLASS.testVirtSubDPArray(arr)) );

// ********************************************
// access default property from
// public method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'public' method of sub class", arr,
                (DYNEXTDCLASS.pubSubSetDPArray(arr), DYNEXTDCLASS.pubSubGetDPArray()) );

// ********************************************
// access default property from
// private method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'private' method of sub class", arr,
             (DYNEXTDCLASS.testPrivSubDPArray(arr)) );

// ********************************************
// access default property from
// public final method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'public final' method of sub class", arr,
             (DYNEXTDCLASS.pubFinSubSetDPArray(arr), DYNEXTDCLASS.pubFinSubGetDPArray()) );

// ********************************************
// access default property from
// public virtual method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'public virtual' method of sub class", arr,
             (DYNEXTDCLASS.pubVirtSubSetDPArray(arr), DYNEXTDCLASS.pubVirtSubGetDPArray()) );

// ********************************************
// access default property from
// private final method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'private final' method of sub class", arr,
             (DYNEXTDCLASS.testPrivFinSubDPArray(arr)) );

// ********************************************
// access default property from
// private virtual method in sub class
// ********************************************

DYNEXTDCLASS = new PubFinExtDefaultClass();
Assert.expectEq( "access 'default' property from 'private virtual' method of sub class", arr,
             (DYNEXTDCLASS.testPrivVirtSubDPArray(arr)) );

            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
