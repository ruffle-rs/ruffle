/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.1.2.7";
//     var VERSION = "ECMA_1";
//     var TITLE   = "isFinite( x )";

    var BUGNUMBER= "77391";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "isFinite.length",      1,                  isFinite.length );
    //array[item++] = Assert.expectEq(  "isFinite.length = null; isFinite.length",   1,      eval("isFinite.length=null; isFinite.length") );

    var thisError:String = "no error";
    try
    {
        isFinite.length=null;
    }
    catch(e:ReferenceError)
    {
        thisError = e.toString();
    }
    finally
    {
        array[item++] = Assert.expectEq( "isFinite.length = null", "ReferenceError: Error #1074", Utils.referenceError(thisError));
    }
    array[item++] = Assert.expectEq(  "delete isFinite.length",                    false,  delete isFinite.length );
    //array[item++] = Assert.expectEq(  "delete isFinite.length; isFinite.length",   1,      eval("delete isFinite.length; isFinite.length") );
    delete isFinite.length;
    array[item++] = Assert.expectEq(  "delete isFinite.length; isFinite.length",   1, isFinite.length);

    var MYPROPS='';
    for ( var p in isFinite ) {
       MYPROPS += p;
    }


    array[item++] = Assert.expectEq(  "var MYPROPS='', for ( var p in isFinite ) { MYPROPS+= p }, MYPROPS",    "", MYPROPS );

    array[item++] = Assert.expectEq(   "isFinite()",           false,              isFinite() );
    array[item++] = Assert.expectEq(  "isFinite( null )",      true,              isFinite(null) );
    array[item++] = Assert.expectEq(  "isFinite( void 0 )",    false,             isFinite(void 0) );
    array[item++] = Assert.expectEq(  "isFinite( false )",     true,              isFinite(false) );
    array[item++] = Assert.expectEq(  "isFinite( true)",       true,              isFinite(true) );
    array[item++] = Assert.expectEq(  "isFinite( ' ' )",       true,              isFinite( " " ) );

    array[item++] = Assert.expectEq(  "isFinite( new Boolean(true) )",     true,   isFinite(new Boolean(true)) );
    array[item++] = Assert.expectEq(  "isFinite( new Boolean(false) )",    true,   isFinite(new Boolean(false)) );

    array[item++] = Assert.expectEq(  "isFinite( 0 )",        true,              isFinite(0) );
    array[item++] = Assert.expectEq(  "isFinite( 1 )",        true,              isFinite(1) );
    array[item++] = Assert.expectEq(  "isFinite( 2 )",        true,              isFinite(2) );
    array[item++] = Assert.expectEq(  "isFinite( 3 )",        true,              isFinite(3) );
    array[item++] = Assert.expectEq(  "isFinite( 4 )",        true,              isFinite(4) );
    array[item++] = Assert.expectEq(  "isFinite( 5 )",        true,              isFinite(5) );
    array[item++] = Assert.expectEq(  "isFinite( 6 )",        true,              isFinite(6) );
    array[item++] = Assert.expectEq(  "isFinite( 7 )",        true,              isFinite(7) );
    array[item++] = Assert.expectEq(  "isFinite( 8 )",        true,              isFinite(8) );
    array[item++] = Assert.expectEq(  "isFinite( 9 )",        true,              isFinite(9) );

    array[item++] = Assert.expectEq(  "isFinite( '0' )",        true,              isFinite('0') );
    array[item++] = Assert.expectEq(  "isFinite( '1' )",        true,              isFinite('1') );
    array[item++] = Assert.expectEq(  "isFinite( '2' )",        true,              isFinite('2') );
    array[item++] = Assert.expectEq(  "isFinite( '3' )",        true,              isFinite('3') );
    array[item++] = Assert.expectEq(  "isFinite( '4' )",        true,              isFinite('4') );
    array[item++] = Assert.expectEq(  "isFinite( '5' )",        true,              isFinite('5') );
    array[item++] = Assert.expectEq(  "isFinite( '6' )",        true,              isFinite('6') );
    array[item++] = Assert.expectEq(  "isFinite( '7' )",        true,              isFinite('7') );
    array[item++] = Assert.expectEq(  "isFinite( '8' )",        true,              isFinite('8') );
    array[item++] = Assert.expectEq(  "isFinite( '9' )",        true,              isFinite('9') );

    array[item++] = Assert.expectEq(  "isFinite( 0x0a )",    true,                 isFinite( 0x0a ) );
    array[item++] = Assert.expectEq(  "isFinite( 0xaa )",    true,                 isFinite( 0xaa ) );
    array[item++] = Assert.expectEq(  "isFinite( 0x0A )",    true,                 isFinite( 0x0A ) );
    array[item++] = Assert.expectEq(  "isFinite( 0xAA )",    true,                 isFinite( 0xAA ) );

    array[item++] = Assert.expectEq(  "isFinite( '0x0a' )",    true,               isFinite( "0x0a" ) );
    array[item++] = Assert.expectEq(  "isFinite( '0xaa' )",    true,               isFinite( "0xaa" ) );
    array[item++] = Assert.expectEq(  "isFinite( '0x0A' )",    true,               isFinite( "0x0A" ) );
    array[item++] = Assert.expectEq(  "isFinite( '0xAA' )",    true,               isFinite( "0xAA" ) );

    array[item++] = Assert.expectEq(  "isFinite( 077 )",       true,               isFinite( 077 ) );
    array[item++] = Assert.expectEq(  "isFinite( '077' )",     true,               isFinite( "077" ) );

    array[item++] = Assert.expectEq(  "isFinite( new String('Infinity') )",        false,      isFinite(new String("Infinity")) );
    array[item++] = Assert.expectEq(  "isFinite( new String('-Infinity') )",       false,      isFinite(new String("-Infinity")) );

    array[item++] = Assert.expectEq(  "isFinite( 'Infinity' )",        false,      isFinite("Infinity") );
    array[item++] = Assert.expectEq(  "isFinite( '-Infinity' )",       false,      isFinite("-Infinity") );
    array[item++] = Assert.expectEq(  "isFinite( Number.POSITIVE_INFINITY )",  false,  isFinite(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "isFinite( Number.NEGATIVE_INFINITY )",  false,  isFinite(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(  "isFinite( Number.NaN )",                false,  isFinite(Number.NaN) );

    array[item++] = Assert.expectEq(  "isFinite( Infinity )",  false,  isFinite(Infinity) );
    array[item++] = Assert.expectEq(  "isFinite( -Infinity )",  false,  isFinite(-Infinity) );
    array[item++] = Assert.expectEq(  "isFinite( NaN )",                false,  isFinite(NaN) );


    array[item++] = Assert.expectEq(  "isFinite( Number.MAX_VALUE )",          true,  isFinite(Number.MAX_VALUE) );
    array[item++] = Assert.expectEq(  "isFinite( Number.MIN_VALUE )",          true,  isFinite(Number.MIN_VALUE) );

    return ( array );
}
