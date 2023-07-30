/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "11.1.4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array Initialiser";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    Array_One = []



    array[item++] = Assert.expectEq(   
                                    "typeof Array_One",
                                    "object",
                                    typeof Array_One );

    array[item++] = Assert.expectEq(   
                                    "Array_One=[]; Array_One.getClass = Object.prototype.toString; Array_One.getClass()",
                                    "[object Array]",
                                    (Array_One.getClass = Object.prototype.toString, Array_One.getClass() ) );

  array[item++] = Assert.expectEq(   
                                    "Array_One = []; Array_One.toString == Array.prototype.toString",
                                    true,
                                    (Array_One.toString == Array.prototype.toString ) );

   array[item++] = Assert.expectEq(   
                                    "Array_One.length",
                                    0,
                                    Array_One.length );

  Array_Two = [1,2,3]


  array[item++] = Assert.expectEq(   
                                    "Array_Two",
                                    "1,2,3",
                                    Array_Two+"" );



  array[item++] = Assert.expectEq(   
                                    "typeof Array_Two",
                                    "object",
                                    typeof Array_Two);

  array[item++] = Assert.expectEq(   
                                    "Array_Two=[1,2,3]; arr.getClass = Object.prototype.toString; arr.getClass()",
                                    "[object Array]",
                                    (Array_Two.getClass = Object.prototype.toString, Array_Two.getClass() ) );

  array[item++] = Assert.expectEq(   
                                    "Array_Two.toString == Array.prototype.toString",
                                    true,
                                    (Array_Two.toString == Array.prototype.toString ) );

  array[item++] = Assert.expectEq(   
                                    "Array_Two.length",
                                    3,
                                    Array_Two.length );

   Array_Three = [12345]

  array[item++] = Assert.expectEq(   
                                    "typeof Array_Three",
                                    "object",
                                    typeof Array_Three );

  array[item++] = Assert.expectEq(   
                                    "Array_Three=[12345]; Array_Three.getClass = Object.prototype.toString; Array_Three.getClass()",
                                    "[object Array]",
                                    (Array_Three.getClass = Object.prototype.toString, Array_Three.getClass() ) );

  Array_Four = [1,2,3,4,5]

  array[item++] = Assert.expectEq(   
                                    "Array_Four.toString == Array.prototype.toString",
                                    true,
                                    (Array_Four.toString == Array.prototype.toString ) );

   Array_Five = [,2,3,4,5]

    array[item++] = Assert.expectEq(   
                                    "Array_Five.length",
                                    5,
                                    Array_Five.length );

   array[item++] = Assert.expectEq(   
                                    "Array_Five[1]",
                                    2,
                                    Array_Five[1] );

   Array_Six = [1,2,3,4,5,6,7,8,9,10,11,12,13,,15,16,17,18,19,20,21,22,23,24,25]

   array[item++] = Assert.expectEq(   
                                    "Array_Six.length",
                                    25,
                                    Array_Six.length );

   array[item++] = Assert.expectEq(   
                                    "Array_Six[14]",
                                    15,
                                    Array_Six[14] );

   Array_Seven = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,]

   array[item++] = Assert.expectEq(   
                                    "Array_Seven.length",
                                    32,
                                    Array_Seven.length );
  Array_Eight=[,,,,,,,,,,,,,,,]

   array[item++] = Assert.expectEq(   
                                    "Array_Eight.length",
                                    15,
                                    Array_Eight.length );

   Array_Nine = [,2,3,4,5,6,7,8,9,10,11,,13,14,15,16,17,18,19,,21,22,23,24,25,26,27,28,29,30,31,32,]

   array[item++] = Assert.expectEq(   
                                    "Array_Nine.length",
                                    32,
                                    Array_Nine.length );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[1]",
                                    2,
                                    Array_Nine[1] );

  array[item++] = Assert.expectEq(   
                                    "Array_Nine[0]",
                                    undefined,
                                    Array_Nine[0] );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[11]",
                                    undefined,
                                    Array_Nine[11] );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[12]",
                                    13,
                                    Array_Nine[12] );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[19]",
                                    undefined,
                                    Array_Nine[19] );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[20]",
                                    21,
                                    Array_Nine[20] );

   array[item++] = Assert.expectEq(   
                                    "Array_Nine[32]",
                                    undefined,
                                    Array_Nine[32] );

   var Array_Ten:Array = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

   array[item++] = Assert.expectEq(   
                                    "Multi dimensional array",
                                    6,
                                    Array_Ten[1][2] );

   var obj = new Object();
   obj.prop1 = "Good";
   obj.prop2 = "one";
   for (j in obj){

       var myvar = obj[j];
       if (myvar=="one"){
          break;
       }
       //print(myvar);
   }

   array[item++] = Assert.expectEq(   "Using array initializers to dynamically set and retrieve values of an object","one",myvar );






    return ( array );
}

