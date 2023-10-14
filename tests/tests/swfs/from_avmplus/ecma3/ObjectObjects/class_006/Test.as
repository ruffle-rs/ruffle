/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
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

package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import flash.system.Capabilities;
import com.adobe.test.Assert;

// var SECTION = "class_006";
// var VERSION = "";
// var TITLE   = "Testing the internal [[Class]] property of objects";
// var bug = "(none)";

var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var status = '';
    var actual = '';
    var expect= '';
        var k = new Function();
        
        
    

    // new Function() will be dropped in ecma4, will return undefined
    // new Function() has been replaced by function() {}
    status = 'new Function()';
    actual = getJSClass(new Function());
    expect = 'Function';
    array[item++] = Assert.expectEq( status, expect, actual);

    
    
    return array;
}
var cnNoObject = 'Unexpected Error!!! Parameter to this function must be an object';
var cnNoClass = 'Unexpected Error!!! Cannot find Class property';
var cnObjectToString = Object.prototype.toString;
var k = new Function();


// checks that it's safe to call findType()
function getJSType(obj)
{
  if (isObject(obj))
    return (findType(obj));
  return cnNoObject;
}

// checks that it's safe to call findType()
function getJSClass(obj)
{
  if (isObject(obj))
    return findClass(findType(obj));
  return cnNoObject;
}


function findType(obj)
{    // TODO: REVIEW AS4 CONVERSION ISSUE 
  var cnObjectToString = Object.prototype.toString;
  return ((cnObjectToString.apply(obj)).substring(0,16)+"]");
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


function isObject(obj)
{
  return obj instanceof Object;
}




