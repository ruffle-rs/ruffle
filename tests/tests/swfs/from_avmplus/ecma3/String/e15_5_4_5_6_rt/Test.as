/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.5.4.5-6";
//     var VERSION = "ECMA_2";
//     var TITLE   = "String.prototype.charCodeAt";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var thisError="no error";
    var obj = true;
    var s = ''
    
    var origBooleanCharCodeAt = Boolean.prototype.charCodeAt;
    Boolean.prototype.charCodeAt= String.prototype.charCodeAt;
    try{
    
        obj.__proto__.charAt = String.prototype.charAt;
        for ( var i = 0; i < 4; i++ )
            s+= String.fromCharCode( obj.charCodeAt(i) );
    }catch(eP0:ReferenceError){
        thisError=eP0.toString();
    }
    array[item++] = Assert.expectEq( "var obj = true; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s","ReferenceError: Error #1069", Utils.referenceError(thisError));
     

    /*var obj = true;
    obj.__proto__.charCodeAt = String.prototype.charCodeAt;
    var s = '';
    for ( var i = 0; i < 4; i++ )
        s+= String.fromCharCode( obj.charCodeAt(i) );

    array[item++] = Assert.expectEq( 
                                  "var obj = true; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s",
                                  "true",
                                  s);*/

    thisError="no error";
    var obj = 1234;
    var s = '';
    try{
    
        obj.__proto__.charAt = String.prototype.charAt;
        for ( var i = 0; i < 4; i++ )
            s+= String.fromCharCode( obj.charCodeAt(i) );
    }catch(eP1:Error){
        thisError=eP1.toString();
    }
    array[item++] = Assert.expectEq( "var obj = 1234; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s", "ReferenceError: Error #1069",Utils.referenceError(thisError));
     

    /*obj = 1234;
    obj.__proto__.charCodeAt = String.prototype.charCodeAt;
    s = '';
    for ( var i = 0; i < 4; i++ )
        s+= String.fromCharCode( obj.charCodeAt(i) );

    array[item++] = Assert.expectEq( 
                                  "var obj = 1234; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s",
                                  "1234",
                                   s);*/
    thisError="no error";
    var obj = 'hello';
    var s = '';
    try{
    
        obj.__proto__.charAt = String.prototype.charAt;
        for ( var i = 0; i < 4; i++ )
            s+= String.fromCharCode( obj.charCodeAt(i) );
    }catch(eP2:Error){
        thisError=eP2.toString();
    }
    array[item++] = Assert.expectEq( "var obj = 1234; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s", "ReferenceError: Error #1069",Utils.referenceError(thisError));
     

   /* obj = 'hello';
    obj.__proto__.charCodeAt = String.prototype.charCodeAt;
    s = '';
    for ( var i = 0; i < 5; i++ )
        s+= String.fromCharCode( obj.charCodeAt(i) );

    array[item++] = Assert.expectEq( 
                                  "var obj = 'hello'; obj.__proto__.charCodeAt = String.prototype.charCodeAt; var s = ''; for ( var i = 0; i < 5; i++ ) s+= String.fromCharCode( obj.charCodeAt(i) ); s",
                                  "hello",
                                  s );*/

    var myvar = new String(true);
    var s = '';
    for ( var i = 0; i < 4; i++ )
        s+= String.fromCharCode( myvar.charCodeAt(i))
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String(true); var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( myvar.charCodeAt(i) ); s",
                                  "true",
                                  s);

    var myvar = new String(1234);
    var s = '';
    for ( var i = 0; i < 4; i++ )
        s+= String.fromCharCode( myvar.charCodeAt(i))
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String(1234); var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( myvar.charCodeAt(i) ); s",
                                  "1234",
                                  s);

    var myvar = new String('hello');
    var s = '';
    for ( var i = 0; i < myvar.length; i++ )
        s+= String.fromCharCode( myvar.charCodeAt(i))
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String('hello'); var s = ''; for ( var i = 0; i < 4; i++ ) s+= String.fromCharCode( myvar.charCodeAt(i) ); s",
                                  "hello",
                                  s);

    var myvar = new String('hello');
    var s = '';
    s = myvar.charCodeAt(-1);
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String('hello'); var s = myvar.charCodeAt(-1)",NaN,s);

    var myvar = new String(1234);
    var s = '';
    s = myvar.charCodeAt(0);
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String(1234); var s = myvar.charCodeAt(0)",49,s);

    var myvar = new String(1234);
    var s = '';
    s = String.fromCharCode(myvar.charCodeAt());
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String(1234); var s = String.fromCharCode(myvar.charCodeAt())","1",s);

    var myvar = new String(1234);
    var s = '';
    s = myvar.charCodeAt(5);
    
    array[item++] = Assert.expectEq( 
                                  "var myvar = new String(1234); var s = myvar.charCodeAt(5)",NaN,s);

    var myobj = new Object();
    myobj.length = 5
    myobj.charCodeAt = String.prototype.charCodeAt;
    myobj[0]='h';
    myobj[1]='e';
    myobj[2]='l';
    myobj[3]='l';
    myobj[4]='o';
    array[item++] = Assert.expectEq( 
                                  "var myobj = new Object();myobj.charCodeAt = String.prototype.charCodeAt;  myobj.charCodeAt(4)",101,myobj.charCodeAt(4));


    Boolean.prototype.charCodeAt= origBooleanCharCodeAt;

    return (array );
}
