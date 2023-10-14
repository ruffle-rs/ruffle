/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "15.2.1.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Object( value )";


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

    var NULL_OBJECT = Object(null);
    
    var expectedError = 1056;
    if (true) {     // TODO: REVIEW AS4 CONVERSION ISSUE 
        expectedError = 1037;
    }


           array[item++] = Assert.expectEq(  "Object(null).valueOf()", NULL_OBJECT,NULL_OBJECT.valueOf());

           array[item++] = Assert.expectEq(  "typeof Object(null)",       "object",               typeof (Object(null)) );

           array[item++] = Assert.expectEq(  "Object(null).constructor.prototype", "[object Object]",Object(null).constructor.prototype+"");





   var UNDEFINED_OBJECT = Object( void 0 );

           array[item++] = Assert.expectEq(  "Object(void 0).valueOf()", UNDEFINED_OBJECT,UNDEFINED_OBJECT.valueOf());



    array[item++] = Assert.expectEq(  "typeof Object(void 0)",       "object",               typeof (Object(void 0)) );

           array[item++] = Assert.expectEq(  "Object(void 0).constructor.prototype", "[object Object]",(Object(void 0)).constructor.prototype+"");


    var UNDEFINED_OBJECT2 = Object(undefined);

           array[item++] = Assert.expectEq(  "Object(undefined).valueOf", UNDEFINED_OBJECT2,UNDEFINED_OBJECT2.valueOf());



    array[item++] = Assert.expectEq(  "typeof Object(undefined)",       "object",               typeof (Object(undefined)) );

           array[item++] = Assert.expectEq(  "Object(undefined).constructor.prototype", "[object Object]",(Object(undefined)).constructor.prototype+"");


    array[item++] = Assert.expectEq(  "Object(true).valueOf()",    true,                   (Object(true)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(true)",       "boolean",               typeof Object(true) );
    thisError = "no error";
    try{
       var MYOB = Object(true);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e4:Error){
           thisError=e4.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(true), MYOB.toString = Object.prototype.toString, MYOB.toString()", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


  array[item++] = Assert.expectEq(  "Object(false).valueOf()",    false,                  (Object(false)).valueOf() );
  array[item++] = Assert.expectEq(  "typeof Object(false)",      "boolean",               typeof Object(false) );


  thisError = "no error";
    try{
       var MYOB = Object(false);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e5:Error){
           thisError=e5.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(false), MYOB.toString = Object.prototype.toString, MYOB.toString()", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


  array[item++] = Assert.expectEq(  "Object(0).valueOf()",       0,                      (Object(0)).valueOf() );
  array[item++] = Assert.expectEq(  "typeof Object(0)",          "number",               typeof Object(0) );
  thisError = "no error";
    try{
       var MYOB = Object(0);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e6:Error){
           thisError=e6.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(0), MYOB.toString = Object.prototype.toString, MYOB.toString()", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }




  array[item++] = Assert.expectEq(  "Object(-0).valueOf()",      -0,                     (Object(-0)).valueOf() );
  array[item++] = Assert.expectEq(  "typeof Object(-0)",         "number",               typeof Object(-0) );


  thisError = "no error";
    try{
       var MYOB = Object(-0);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e7:Error){
           thisError=e7.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(-0), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


  array[item++] = Assert.expectEq(  "Object(1).valueOf()",       1,                      (Object(1)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(1)",          "number",               typeof Object(1) );
   thisError = "no error";
    try{
       var MYOB = Object(1);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e8:Error){
           thisError=e8.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(1), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }



    array[item++] = Assert.expectEq(  "Object(-1).valueOf()",      -1,                     (Object(-1)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(-1)",         "number",               typeof Object(-1) );

    thisError = "no error";
    try{
       var MYOB = Object(-1);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e9:Error){
           thisError=e9.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(-1), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


    array[item++] = Assert.expectEq(  "Object(Number.MAX_VALUE).valueOf()",    1.7976931348623157e308,         (Object(Number.MAX_VALUE)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.MAX_VALUE)",       "number", typeof Object(Number.MAX_VALUE) );

    thisError = "no error";
    try{
       var MYOB = Object(Number.MAX_VALUE);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e10:Error){
           thisError=e10.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(Number.MAX_VALUE), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }

    array[item++] = Assert.expectEq(  "Object(Number.MIN_VALUE).valueOf()",     5e-324,           (Object(Number.MIN_VALUE)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.MIN_VALUE)",       "number",         typeof Object(Number.MIN_VALUE) );

    thisError = "no error";
    try{
       var MYOB = Object(Number.MIN_VALUE);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e11:Error){
           thisError=e11.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(Number.MIN_VALUE), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


    array[item++] = Assert.expectEq(  "Object(Number.POSITIVE_INFINITY).valueOf()",    Number.POSITIVE_INFINITY,       (Object(Number.POSITIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.POSITIVE_INFINITY)",       "number",                       typeof Object(Number.POSITIVE_INFINITY) );

    thisError = "no error";
    try{
       var MYOB = Object(Number.POSITIVE_INFINITY);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e12:Error){
           thisError=e12.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(Number.POSITIVE_INFINITY), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


    array[item++] = Assert.expectEq(  "Object(Number.NEGATIVE_INFINITY).valueOf()",    Number.NEGATIVE_INFINITY,       (Object(Number.NEGATIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.NEGATIVE_INFINITY)",       "number",            typeof Object(Number.NEGATIVE_INFINITY) );

    thisError = "no error";
    try{
       var MYOB = Object(Number.NEGATIVE_INFINITY);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e13:Error){
           thisError=e13.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(Number.NEGATIVE_INFINITY), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }

  array[item++] = Assert.expectEq(  "Object(Number.NaN).valueOf()",      Number.NaN,                (Object(Number.NaN)).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object(Number.NaN)",         "number",                  typeof Object(Number.NaN) );
    thisError = "no error";
    try{
       var MYOB = Object(Number.NaN);
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e14:Error){
           thisError=e14.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(Number.NaN), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }



    array[item++] = Assert.expectEq(  "Object('a string').valueOf()",      "a string",         (Object("a string")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('a string')",         "string",           typeof (Object("a string")) );

    thisError = "no error";
    try{
       var MYOB = Object('a string');
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e15:Error){
           thisError=e15.toString();
       }finally{
         array[item++] = Assert.expectEq(  "var MYOB = Object('a string'), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }

    array[item++] = Assert.expectEq(  "Object('').valueOf()",              "",                 (Object("")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('')",                 "string",           typeof (Object("")) );

    thisError = "no error";
    try{
       var MYOB = Object('');
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e16:Error){
           thisError=e16.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object(''), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


    array[item++] = Assert.expectEq(  "Object('\\r\\t\\b\\n\\v\\f').valueOf()",   "\r\t\b\n\v\f",   (Object("\r\t\b\n\v\f")).valueOf() );
    array[item++] = Assert.expectEq(  "typeof Object('\\r\\t\\b\\n\\v\\f')",      "string",           typeof (Object("\\r\\t\\b\\n\\v\\f")) );

    thisError = "no error";
    try{
       var MYOB = Object('\\r\\t\\b\\n\\v\\f');
       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e17:Error){
           thisError=e17.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object('\\r\\t\\b\\n\\v\\f'), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }


    array[item++] = Assert.expectEq(   "Object( '\\\'\\\"\\' ).valueOf()",      "\'\"\\",          (Object("\'\"\\")).valueOf() );
    array[item++] = Assert.expectEq(   "typeof Object( '\\\'\\\"\\' )",        "string",           typeof Object("\'\"\\") );

    var MYOB = Object('\\\'\\\"\\' );
    thisError = "no error";
    try{

       MYOB.toString = Object.prototype.toString;
       MYOB.toString()
       }catch(e18:Error){
           thisError=e18.toString();
       }finally{
           array[item++] = Assert.expectEq(  "var MYOB = Object('\\\'\\\"\\' ), MYOB.toString", Utils.REFERENCEERROR+expectedError,Utils.referenceError(thisError));
        }





    return ( array );
}
