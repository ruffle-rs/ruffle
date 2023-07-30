/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.2.4.3";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Object.prototype.valueOf()";


    var testcases = getTestCases();

   // TODO: REVIEW AS4 CONVERSION ISSUE 
 // Adding this function getJSClass directly to file rather than in Utils

 function getJSClass(obj)
{
  if (isObject(obj))
    return findClass(findType(obj));
  return cnNoObject;
}
function isObject(obj)
{
  return obj instanceof Object;
}

function findType(obj)
{
  var cnObjectToString = Object.prototype.toString;
  return cnObjectToString.apply(obj);
}
// given '[object Number]',  return 'Number'
function findClass(sType)
{
  var re =  /^\[.*\s+(\w+)\s*\]$/;
  var a = sType.match(re);

  if (a && a[1])
    return a[1];
  return cnNoClass;
}

function getTestCases() {
    var array = new Array();
    var item = 0;

    // save
    var origNumProto = Number.prototype.valueOf;
    var origBoolProto = Boolean.prototype.valueOf;

    var myarray = new Array();
    myarray.valueOf = Object.prototype.valueOf;
    //Boolean.prototype.valueOf=Object.prototype.valueOf;
    //var myboolean = new Boolean();
    var myfunction = function() {};
    myfunction.valueOf = Object.prototype.valueOf;
    var myobject = new Object();
    myobject.valueOf = Object.prototype.valueOf;
   var mymath = Math;
    //mymath.valueOf = Object.prototype.valueOf;
    var mydate = new Date();
    //mydate.valueOf = Object.prototype.valueOf;
   Number.prototype.valueOf=Object.prototype.valueOf;
    var mynumber = new Number();
    //String.prototype.valueOf=Object.prototype.valueOf;
    var mystring = new String();

    array[item++] = Assert.expectEq(   "Object.prototype.valueOf.length",      0,      Object.prototype.valueOf.length );

    array[item++] = Assert.expectEq( 
                                 "myarray = new Array(); myarray.valueOf = Object.prototype.valueOf; myarray.valueOf()",
                                 myarray,
                                 myarray.valueOf() );
    Boolean.prototype.valueOf=Object.prototype.valueOf;
    var myboolean:Boolean = new Boolean();
    var thisError:String="no error";
    try{

       myboolean.valueOf = Object.prototype.valueOf;
       myboolean.valueOf();
    }catch(e:Error){
       thisError=e.toString();
    }finally{//print(thisError);
        
        var expectedError = 1056;
        if (true) {     // TODO: REVIEW AS4 CONVERSION ISSUE 
            expectedError = 1037;
        }
        
        array[item++] = Assert.expectEq( 
                                 "myboolean = new Boolean(); myboolean.valueOf = Object.prototype.valueOf; myboolean.valueOf()",
                                 Utils.REFERENCEERROR+expectedError,
                                 Utils.referenceError(thisError) );
     }
    /*array[item++] = Assert.expectEq( 
                                 "myboolean = new Boolean(); myboolean.valueOf = Object.prototype.valueOf; myboolean.valueOf()",
                                 myboolean,
                                 myboolean.valueOf() );*/
    array[item++] = Assert.expectEq( 
                                 "myfunction = function() {}; myfunction.valueOf = Object.prototype.valueOf; myfunction.valueOf()",
                                 myfunction,
                                 myfunction.valueOf() );

    array[item++] = Assert.expectEq( 
                                 "myobject = new Object(); myobject.valueOf = Object.prototype.valueOf; myobject.valueOf()",
                                 myobject,
                                 myobject.valueOf() );

    try{
       mymath.valueOf = Object.prototype.valueOf;
       mymath.valueOf();
    }catch(e1:Error){
       thisError=e1.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq( 
        "mymath = Math; mymath.valueOf = Object.prototype.valueOf;mymath.valueOf()",
        Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError) );
     }
   /* array[item++] = Assert.expectEq( 
                                 "mymath = Math; mymath.valueOf = Object.prototype.valueOf; mymath.valueOf()",
                                 mymath,
                                 mymath.valueOf() );*/
    try{
       mynumber.valueOf = Object.prototype.valueOf;
       mynumber.valueOf();
    }catch(e2:ReferenceError){
       thisError=e2.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq( 
        "mynumber = new Number(); mynumber.valueOf = Object.prototype.valueOf; mynumber.valueOf()",
        Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError) );
     }

    /*array[item++] = Assert.expectEq( 
                                 "mynumber = new Number(); mynumber.valueOf = Object.prototype.valueOf; mynumber.valueOf()",
                                 mynumber,
                                 mynumber.valueOf() );*/

    try{
       mystring.valueOf = Object.prototype.valueOf;
       mystring.valueOf();
    }catch(e3:Error){
       thisError=e3.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq( 
        "mystring = new String(); mystring.valueOf = Object.prototype.valueOf; mystring.valueOf()",
        Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError) );
     }
   /* array[item++] = Assert.expectEq( 
                                 "mystring = new String(); mystring.valueOf = Object.prototype.valueOf; mystring.valueOf()",
                                 mystring,
                                 mystring.valueOf() );*/

    try{
       mydate.valueOf = Object.prototype.valueOf;
       mydate.valueOf();
    }catch(e4:Error){
       thisError=e4.toString();
    }finally{//print(thisError);
        array[item++] = Assert.expectEq( 
        "mydate = new Date(); mydate.valueOf = Object.prototype.valueOf; mydate.valueOf()",
        Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError) );
     }
    /*array[item++] = Assert.expectEq( 
                                 "mydate = new Date(); mydate.valueOf = Object.prototype.valueOf; mydate.valueOf()",
                                 mydate,
                                 mydate.valueOf());*/

    // restore
    Number.prototype.valueOf = origNumProto;
    Boolean.prototype.valueOf = origBoolProto;

    return ( array );
}
