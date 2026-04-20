/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}
import com.adobe.test.Assert;

//     var SECTION = "11_2_4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Argument List";


    var testcases = getTestCases();


  class MyClass{}
function getTestCases() {
    var array = new Array();
    var item = 0;

    //Empty argument list

    var arr = new Array();

    array[item++] = Assert.expectEq(
                                    "arr.length",
                                    0,
                                    arr.length );

    trace(arr);

    array[item++] = Assert.expectEq(
                                    "Returning empty list of value of arr",
                                    '',
                                    arr.toString() );


   function MyFunction():String{
       return "Hi!";
   }


    array[item++] = Assert.expectEq(
                                    "MyFunction",
                                    "Hi!",
                                    MyFunction() );



   //Argument list with more arguments

    var arr1 = new Array(1,2,3,4,5);

    array[item++] = Assert.expectEq(
                                    "arr1.length",
                                    5,
                                    arr1.length );

    function MyFunction2(a,b,c,d,e):Number{
    return a+b+c+d+e;
    }

    array[item++] = Assert.expectEq(
                                    "MyFunction2 with 5 arguments",
                                    15,
                                    MyFunction2(1,2,3,4,5));
   var myvar1:Number = 1;
   var myvar2:Number =10;
   function foo():Number{
   myvar1 = myvar1+myvar2;
   return myvar1;
   }

   function goo():Number{
   myvar2 = myvar2*10;
   return myvar2;
   }

   function MyFunction3(d,e,f):Number{
   return f;
   }

   array[item++] = Assert.expectEq(
                                    "MyFunction3 with 3 arguments",
                                    111,
                                    MyFunction3(foo(),goo(),myvar1+myvar2));

   var arr2 = new Array(foo(),goo(),myvar1+myvar2);

   array[item++] = Assert.expectEq(
                                    "arr2.length",
                                    3,
                                    arr2.length );

   array[item++] = Assert.expectEq(
                                    "arr2[0]",
                                    111,
                                    arr2[0] );

   array[item++] = Assert.expectEq(
                                    "arr2[1]",
                                    1000,
                                    arr2[1] );

   array[item++] = Assert.expectEq(
                                    "arr2[2]",
                                    1111,
                                    arr2[2] );


  var myclass = new MyClass();

  function MyFunction4(a,b,c,d,e,f):void{}

  array[item++] = Assert.expectEq(
                                    "MyFunction3 with 3 arguments",
                                    11111,
                                    MyFunction3(foo(),goo(),myvar1+myvar2));

  var arr3 = new Array(1,"string",foo(),[1,2,3],true);

  array[item++] = Assert.expectEq(
                                    "arr3.length",
                                    5,
                                    arr3.length );

  array[item++] = Assert.expectEq(
                                    "arr3[0]",
                                    1,
                                    arr3[0] );

   array[item++] = Assert.expectEq(
                                    "arr3[1]",
                                    "string",
                                    arr3[1] );

   array[item++] = Assert.expectEq(
                                    "arr3[2]",
                                    11111,
                                    arr3[2] );

   array[item++] = Assert.expectEq(
                                    "arr3[3]",
                                    "1,2,3",
                                    arr3[3]+"" );



   array[item++] = Assert.expectEq(
                                    "arr3[4]",
                                    true,
                                    arr3[4]);


   var k:Number;
   var l:String;
   var m:Array;
   var p:Number;
   var n:Boolean;
   var q:String;
   var r:Number;

   function MyFunction4(a,b,c,d,e,g,h):String{
   k=a;
   l=b;
   m=c;
   n=e;
   p=d;
   q=g;
   r=h;
   return "passed";
   }

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    "passed",
                                    MyFunction4(1,"string",[2,3,4],goo(),false,null,void 0));

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    1,
                                     k);

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    1,
                                     k);
   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    100000,
                                    p);

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    false,
                                    n);

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    null,
                                    q);

   array[item++] = Assert.expectEq(
                                    "Function with arguments of different data types",
                                    NaN,
                                    r);
    return array;
}


function TestFunction() {
    return arguments;
}