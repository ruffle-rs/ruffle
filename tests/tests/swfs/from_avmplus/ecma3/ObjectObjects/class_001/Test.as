/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 14 Mar 2001
 *
 * SUMMARY: Testing the internal [[Class]] property of objects
 * See ECMA-262 Edition 3 13-Oct-1999, Section 8.6.2
 *
 * The getJSClass() function we use is in a utility file, e.g. "shell.js".
 *
 *    Modified:     28th October 2004 (gasingh@macromedia.com)
 *              Removed the occurence of new Function('abc').
 *              This is being changed to function() { abc }.
 *
 */
//-------------------------------------------------------------------------------------------------
// var SECTION = "class_001";
// var VERSION = "";
// var TITLE   = "Testing the internal [[Class]] property of objects";
// var bug = "(none)";

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

    var status = '';
    var actual = '';
    var expect= '';
        var k = new Function();
        
        
    status = 'the global object';
    actual = getJSClass(this);
    expect = 'global';
    array[item++] = Assert.expectEq( status, expect, actual);
    status = 'new Object()';
    actual = getJSClass(new Object());
    expect = 'Object';
    array[item++] = Assert.expectEq( status, expect, actual);

    // new Function() will be dropped in ecma4, will return undefined
    // new Function() has been replaced by function() {}
    /*status = 'new Function()';
    actual = getJSClass(k)+"";
    expect = 'Function';
    array[item++] = Assert.expectEq( status, expect, actual);*/

    status = 'new Array()';
    actual = getJSClass(new Array());
    expect = 'Array';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new String()';
    actual = getJSClass(new String());
    expect = 'String';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new Boolean()';
    actual = getJSClass(new Boolean());
    expect = 'Boolean';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new Number()';
    actual = getJSClass(new Number());
    expect = 'Number';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'Math';
    actual = getJSClass(Math);  // can't use 'new' with the Math object (EMCA3, 15.8)
    expect = 'Math';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new Date()';
    actual = getJSClass(new Date());
    expect = 'Date';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new RegExp()';
    actual = getJSClass(new RegExp());
    expect = 'RegExp';
    array[item++] = Assert.expectEq( status, expect, actual);

    status = 'new Error()';
    actual = getJSClass(new Error());
    expect = 'Error';
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}
