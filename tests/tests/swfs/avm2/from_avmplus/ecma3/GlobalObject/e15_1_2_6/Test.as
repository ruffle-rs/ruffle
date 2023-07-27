/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.1.2.6";
//     var VERSION = "ECMA_1";
//     var TITLE   = "isNaN( x )";

    var BUGNUMBER = "77391";


    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "isNaN.length",      1,                  isNaN.length );

    var MYPROPS='';
    for ( var p in isNaN ) {
       MYPROPS+= p;
    }

    array[item++] = Assert.expectEq(  "var MYPROPS='', for ( var p in isNaN ) { MYPROPS+= p }, MYPROPS", "", MYPROPS );

    var thisError:String = "no error";
    try
    {
        isNaN.length=null;
    }
    catch (e:ReferenceError)
    {
        thisError = e.toString();
    }
    finally
    {
        array[item++] = Assert.expectEq( "isNaN.length = null", "ReferenceError: Error #1074", Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(  "delete isNaN.length",               false,  delete isNaN.length );
    //array[item++] = Assert.expectEq(  "delete isNaN.length; isNaN.length", 1,      eval("delete isNaN.length; isNaN.length") );
    delete isNaN.length;
    array[item++] = Assert.expectEq(  "delete isNaN.length; isNaN.length", 1, isNaN.length);

//    array[item++] = Assert.expectEq(  "isNaN.__proto__",   Function.prototype, isNaN.__proto__ );

    array[item++] = Assert.expectEq(  "isNaN()",           true,               isNaN() );
    array[item++] = Assert.expectEq(  "isNaN( null )",     false,              isNaN(null) );
    array[item++] = Assert.expectEq(  "isNaN( void 0 )",   true,               isNaN(void 0) );
    array[item++] = Assert.expectEq(  "isNaN( true )",     false,              isNaN(true) );
    array[item++] = Assert.expectEq(  "isNaN( false)",     false,              isNaN(false) );
    array[item++] = Assert.expectEq(  "isNaN( ' ' )",      false,              isNaN( " " ) );

    array[item++] = Assert.expectEq(  "isNaN( 0 )",        false,              isNaN(0) );
    array[item++] = Assert.expectEq(  "isNaN( 1 )",        false,              isNaN(1) );
    array[item++] = Assert.expectEq(  "isNaN( 2 )",        false,              isNaN(2) );
    array[item++] = Assert.expectEq(  "isNaN( 3 )",        false,              isNaN(3) );
    array[item++] = Assert.expectEq(  "isNaN( 4 )",        false,              isNaN(4) );
    array[item++] = Assert.expectEq(  "isNaN( 5 )",        false,              isNaN(5) );
    array[item++] = Assert.expectEq(  "isNaN( 6 )",        false,              isNaN(6) );
    array[item++] = Assert.expectEq(  "isNaN( 7 )",        false,              isNaN(7) );
    array[item++] = Assert.expectEq(  "isNaN( 8 )",        false,              isNaN(8) );
    array[item++] = Assert.expectEq(  "isNaN( 9 )",        false,              isNaN(9) );
    array[item++] = Assert.expectEq(  "isNaN( -1 )",       false,              isNaN(-1) );

    array[item++] = Assert.expectEq(  "isNaN( '-1' )",     false,              isNaN(-1) );

    array[item++] = Assert.expectEq(  "isNaN( 1.23 )",     false,              isNaN(1.23) );

    array[item++] = Assert.expectEq(  "isNaN('1.23')",     false,              isNaN(1.23) );

    array[item++] = Assert.expectEq(  "isNaN( -1.23 )",    false,              isNaN(-1.23) );

    array[item++] = Assert.expectEq(  "isNaN( '-1.23' )",  false,              isNaN(-1.23) );

    array[item++] = Assert.expectEq(  "isNaN( '0' )",        false,              isNaN('0') );
    array[item++] = Assert.expectEq(  "isNaN( '1' )",        false,              isNaN('1') );
    array[item++] = Assert.expectEq(  "isNaN( '2' )",        false,              isNaN('2') );
    array[item++] = Assert.expectEq(  "isNaN( '3' )",        false,              isNaN('3') );
    array[item++] = Assert.expectEq(  "isNaN( '4' )",        false,              isNaN('4') );
    array[item++] = Assert.expectEq(  "isNaN( '5' )",        false,              isNaN('5') );
    array[item++] = Assert.expectEq(  "isNaN( '6' )",        false,              isNaN('6') );
    array[item++] = Assert.expectEq(  "isNaN( '7' )",        false,              isNaN('7') );
    array[item++] = Assert.expectEq(  "isNaN( '8' )",        false,              isNaN('8') );
    array[item++] = Assert.expectEq(  "isNaN( '9' )",        false,              isNaN('9') );


    array[item++] = Assert.expectEq(  "isNaN( 0x0a )",    false,              isNaN( 0x0a ) );
    array[item++] = Assert.expectEq(  "isNaN( 0xaa )",    false,              isNaN( 0xaa ) );
    array[item++] = Assert.expectEq(  "isNaN( 0x0A )",    false,              isNaN( 0x0A ) );
    array[item++] = Assert.expectEq(  "isNaN( 0xAA )",    false,              isNaN( 0xAA ) );

    array[item++] = Assert.expectEq(  "isNaN( '0x0a' )",  false,              isNaN( "0x0a" ) );
    array[item++] = Assert.expectEq(  "isNaN( '0xaa' )",  false,              isNaN( "0xaa" ) );
    array[item++] = Assert.expectEq(  "isNaN( '0x0A' )",  false,              isNaN( "0x0A" ) );
    array[item++] = Assert.expectEq(  "isNaN( '0xAA' )",  false,              isNaN( "0xAA" ) );

    array[item++] = Assert.expectEq(  "isNaN( 077 )",     false,              isNaN( 077 ) );
    array[item++] = Assert.expectEq(  "isNaN( '077' )",   false,              isNaN( "077" ) );


    array[item++] = Assert.expectEq(  "isNaN( Number.NaN )",   true,              isNaN(Number.NaN) );
    array[item++] = Assert.expectEq(  "isNaN( Number.POSITIVE_INFINITY )", false,  isNaN(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "isNaN( Number.NEGATIVE_INFINITY )", false,  isNaN(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "isNaN( Number.MAX_VALUE )",         false,  isNaN(Number.MAX_VALUE) );
    array[item++] = Assert.expectEq(  "isNaN( Number.MIN_VALUE )",         false,  isNaN(Number.MIN_VALUE) );

    array[item++] = Assert.expectEq(  "isNaN( NaN )",               true,      isNaN(NaN) );
    array[item++] = Assert.expectEq(  "isNaN( Infinity )",          false,     isNaN(Infinity) );

    array[item++] = Assert.expectEq(  "isNaN( 'Infinity' )",               false,  isNaN("Infinity") );
    array[item++] = Assert.expectEq(  "isNaN( '-Infinity' )",              false,  isNaN("-Infinity") );

    array[item++] = Assert.expectEq(  "isNaN( 'string' )",              true,  isNaN("string") );

    array[item++] = Assert.expectEq(  "isNaN({} )",true,  isNaN({}));

    array[item++] = Assert.expectEq(  "isNaN(undefined)",true,  isNaN(undefined));


    var arr= new Array();
    array[item++] = Assert.expectEq(  "isNaN(arr)",false,  isNaN(arr));

    var obj = new Object();
    array[item++] = Assert.expectEq(  "isNaN(obj)",true,  isNaN(obj));

    var mydate = new Date(0);
    array[item++] = Assert.expectEq(  "isNaN(mydate)",false,  isNaN(mydate));
    return ( array );
}
