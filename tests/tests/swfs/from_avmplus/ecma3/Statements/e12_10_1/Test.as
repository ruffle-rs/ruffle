/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "12.10-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The with statment";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // although the scope chain changes, the this value is immutable for a given
    // execution context.

    var x;
    with( new Number() )
    {
        x = this +'';
    }
    array[item++] = Assert.expectEq( 
                                "with( new Number() ) { this +'' }",
                                "[object global]",
                                 x);
    // the object's functions and properties should override those of the
    // global object.

    var MYOB = new WithObject(true); with (MYOB) { y=parseInt() }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(true); with (MYOB) { parseInt() }",
                        true,
                        y );

    var MYOB = new WithObject(false);
    with (MYOB)
    {
        z = NaN;
    }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(false); with (MYOB) { NaN }",
                        false,
                        z );

    var MYOB = new WithObject(NaN);
    with (MYOB) { r = Infinity }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(NaN); with (MYOB) { Infinity }",
                        Number.NaN,
                         r);

    var MYOB = new WithObject(false); with (MYOB) { }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(false); with (MYOB) { }; Infinity",
                        Number.POSITIVE_INFINITY,
                        Infinity );


    var MYOB = new WithObject(0); with (MYOB) { delete Infinity; }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(0); with (MYOB) { delete Infinity; Infinity }",
                        Number.POSITIVE_INFINITY,
                        Infinity );
     

    // let us leave the with block via a break.

    var MYOB = new WithObject(false); while (true) { with (MYOB) { MYOB = Infinity; break; } }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(false); while (true) { with (MYOB) { Infinity; break; } }",
                        false,
                        MYOB );
    
    var MYOB = new WithObject(true);
    with (MYOB) { MYOB = Infinity;f(); }
    array[item++] = Assert.expectEq(
                        
                        "var MYOB = new WithObject(true); with (MYOB) { Infinity }",
                        true,
                         MYOB);
function f(a,b){}

     

    return ( array );
}
function WithObject( value ) {
    this.prop1 = 1;
    this.prop2 = new Boolean(true);
    this.prop3 = "a string";
    this.value = value;

    // now we will override global functions

    //this.parseInt = new Function( "return this.value" );
    this.parseInt = function(){return this.value;}
    this.NaN = value;
    this.Infinity = value;
    this.unescape = function(){return this.value;}
    this.escape   = function(){return this.value;}
    this.eval     = function(){return this.value;}
    this.parseFloat = function(){return this.value;}
    this.isNaN      = function(){return this.value;}
    this.isFinite   = function(){return this.value;}
}
