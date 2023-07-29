/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
var GLOBAL = "[object global]";
//     var SECTION = "15.2.4.3";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Object.prototype.toLocaleString()";


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

    array[item++] = Assert.expectEq(   "(new Object()).toLocaleString()",    "[object Object]",  (new Object()).toLocaleString() );

    array[item++] = Assert.expectEq(   "myvar = this;  myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            GLOBAL,
                                            (myvar = this,  myvar.toLocaleString = Object.prototype.toLocaleString, myvar.toLocaleString()) );

    // work around for bug 175820
    array[item++] = Assert.expectEq(   "myvar = MyObject; myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            true,
                                            (myvar = MyObject, myvar.toLocaleString = Object.prototype.toLocaleString, myvar.toLocaleString()).match(/\[object Function-[0-9]+\]/) != null ||
                                            (myvar = MyObject, myvar.toLocaleString = Object.prototype.toLocaleString, myvar.toLocaleString())=="[object null]"
                                             );

    array[item++] = Assert.expectEq(   "myvar = new MyObject( true ); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            '[object Object]',
                                            (myvar = new MyObject( true ), myvar.toLocaleString = Object.prototype.toLocaleString, myvar.toLocaleString()) );

    Number.prototype.toLocaleString;
    //Object.prototpye.toLocaleString;
    array[item++] = Assert.expectEq(   "myvar = new Number(0); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "0",
                                            (myvar = new Number(0), myvar.toLocaleString()) );

/*    array[item++] = Assert.expectEq(   "myvar = new String(''); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object String]",
                                            eval("myvar = new String(''); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "myvar = Math; myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object Math]",
                                            eval("myvar = Math; myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "myvar = function() {}; myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object Function]",
                                            eval("myvar = function() {}; myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "myvar = new Array(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object Array]",
                                            eval("myvar = new Array(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "myvar = new Boolean(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object Boolean]",
                                            eval("myvar = new Boolean(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "myvar = new Date(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()",
                                            "[object Date]",
                                            eval("myvar = new Date(); myvar.toLocaleString = Object.prototype.toLocaleString; myvar.toLocaleString()") );

    array[item++] = Assert.expectEq(   "var MYVAR = new Object( this ); MYVAR.toLocaleString()",
                                            GLOBAL,
                                            (MYVAR = new Object( this ), MYVAR.toLocaleString() ) );

    array[item++] = Assert.expectEq(   "var MYVAR = new Object(); MYVAR.toLocaleString()",
                                            "[object Object]",
                                            (MYVAR = new Object(), MYVAR.toLocaleString() ) );

    array[item++] = Assert.expectEq(   "var MYVAR = new Object(void 0); MYVAR.toLocaleString()",
                                            "[object Object]",
                                            (MYVAR = new Object(void 0), MYVAR.toLocaleString() ) );

    array[item++] = Assert.expectEq(   "var MYVAR = new Object(null); MYVAR.toLocaleString()",
                                            "[object Object]",
                                            (MYVAR = new Object(null), MYVAR.toLocaleString() ) );
*/
    return ( array );
}

function MyObject( value ) {
    this.value = function() { return this.value; }
    this.toLocaleString = function() { return this.value+''; }

    this.value = function() {return this.value;}
    this.toLocaleString = function() {return this.value+'';}
}
