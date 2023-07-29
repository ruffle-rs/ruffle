/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 2001-07-10
 *
 * SUMMARY:  Invoking try...catch through Function.call
 * See  http://bugzilla.mozilla.org/show_bug.cgi?id=49286
 *
 * 1) Define a function with a try...catch block in it
 * 2) Invoke the function via the call method of Function
 * 3) Pass bad syntax to the try...catch block
 * 4) We should catch the error!
 */
//-------------------------------------------------------------------------------------------------
//     var SECTION = "regress_49286";
//     var VERSION = "";
    var cnErrorCaught = 'Error caught';
    var cnErrorNotCaught = 'Error NOT caught';
    var cnGoodSyntax = '1==2';
    var cnBadSyntax = '1=2';


//     var TITLE   = "Invoking try...catch through Function.call";
//     var bug = 49286;


    var testcases = getTestCases();
    

function getTestCases() {
    var array = new Array();
    var item = 0;
    var UBound = 0;

    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];


    var obj = new testObject();
    
    status = 'Section A of test: direct call of f';
    actual = f.call(obj);
    expect = cnErrorCaught;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = 'Section B of test: indirect call of f';
    actual = g.call(obj);
    expect = cnErrorCaught;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    return array;
}
/*
function test()
{
  enterFunc ('test');
  printBugNumber (bug);
  printStatus (summary);

  for (var i=0; i<UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
  }

  exitFunc ('test');
}
 */

// An object storing bad syntax as a property -
function testObject()
{
  this.badSyntax = cnBadSyntax;
  this.goodSyntax = cnGoodSyntax;
}


// A function wrapping a try...catch block
function f()
{
  try
  {
    evalXXX(this.badSyntax);
  }
  catch(e)
  {
    return cnErrorCaught;
  }
  return cnErrorNotCaught;
}


// A function wrapping a call to f -
function g()
{
  return f.call(this);
}

/*
function addThis()
{
  statusitems[UBound] = status;
  actualvalues[UBound] = actual;
  expectedvalues[UBound] = expect;
  UBound++;
}
 */
