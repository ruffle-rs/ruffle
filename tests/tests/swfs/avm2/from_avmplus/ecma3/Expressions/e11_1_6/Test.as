/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11.1.6";
//     var VERSION = "ECMA_4";
//     var TITLE   = "The Grouping Operator";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
   
   array[item++] = Assert.expectEq(   "typeof(new Object())","object",typeof(new Object()));

   array[item++] = Assert.expectEq(   "typeof(new Array())","object",typeof(new Array()));

   array[item++] = Assert.expectEq(   "typeof(new Date())","object",typeof(new Date()));

   array[item++] = Assert.expectEq(   "typeof(new Boolean())","boolean",typeof(new Boolean()));

   array[item++] = Assert.expectEq(   "typeof(new String())","string",typeof(new String()));

   array[item++] = Assert.expectEq(   "typeof(new Number())","number",typeof(new Number()));

   array[item++] = Assert.expectEq(   "typeof(Math)","object",typeof(Math));

   array[item++] = Assert.expectEq(   "typeof(function(){})","function",typeof(function(){}));

   array[item++] = Assert.expectEq(   "typeof(this)","object",typeof(this));

   var MyVar:Number=10;

   array[item++] = Assert.expectEq(   "typeof(MyVar)","number",typeof(MyVar));

   array[item++] = Assert.expectEq(   "typeof([])","object",typeof([]));

   array[item++] = Assert.expectEq(   "typeof({})","object",typeof({}));

   array[item++] = Assert.expectEq(   "typeof('')","string",typeof(''));

   var MyArray = new Array(1,2,3);

   delete(MyArray[0])

   array[item++] = Assert.expectEq(   "delete(MyArray[0]);MyArray[0]",undefined,MyArray[0]);

   Number.prototype.foo=10;
   
   delete(Number.prototype.foo);

   array[item++] = Assert.expectEq(   "delete(Number.prototype.foo);",undefined,Number.prototype.foo);

   String.prototype.goo = 'hi';

   delete(String.prototype.goo);

   array[item++] = Assert.expectEq(   "delete(String.prototype.goo);",undefined,String.prototype.goo);

   Date.prototype.mydate=new Date(0);

   delete(Date.prototype.mydate);

   array[item++] = Assert.expectEq(   "delete(Date.prototype.mydate);",undefined,Date.prototype.mydate);

   

  array[item++] = Assert.expectEq(   "delete(new String('hi'));",true,delete(new String('hi')));

  array[item++] = Assert.expectEq(   "delete(new Date(0));",true,delete(new Date(0)));

  array[item++] = Assert.expectEq(   "delete(new Number(10));",true,delete(new Number(10)));

  array[item++] = Assert.expectEq(   "delete(new Object());",true,delete(new Object()));

  var obj = new Object();

  array[item++] = Assert.expectEq(   "delete(obj)  Trying to delete an object of reference type should return false",false,delete(obj));

  var a:Number = 10;
  var b:Number = 20;
  var c:Number = 30;
  var d:Number = 40;

  /*Grouping operators are used to change the normal hierarchy of the mathematical operators, expressions inside paranthesis are calculated first before any other expressions are calculated*/

  array[item++] = Assert.expectEq(   "Grouping operator used in defining the hierarchy of the operators",true,(a+b*c+d)!=((a+b)*(c+d)));

//Grouping operators are used when passing parameters through a function

  function myfunction(a):Number{
     return a*a;
  }

  array[item++] = Assert.expectEq(   "Grouping operator used in passing parameters to a function",4,myfunction(2));

  var a:Number = 1;
  var b:Number = 2;
  function foo() { a += b; }
  function bar() { b *= 10; }

  array[item++] = Assert.expectEq(   "Grouping operator used in evaluating functions and returning the results of an expression",23,(foo(), bar(), a + b));

  
 
  
  


  

   
      return ( array );
}

function MyObject( value ) {
    this.value = function() {return this.value;}
    this.toString = function() {return this.value+'';}
}
