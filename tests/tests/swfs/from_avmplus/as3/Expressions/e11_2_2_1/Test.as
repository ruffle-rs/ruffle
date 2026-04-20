/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "11_2_2_1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The new operator";


    var testcases = getTestCases();


class MyClass{

   var MyFirstNumber:Number=0;
   var MySecondNumber:Number=0;

   public function MyClass(a:Number,b:Number){
       MyFirstNumber = a;
       MySecondNumber = b;

   }

   public function MyNumberOne():Number{
       return MyFirstNumber;
   }

   public function MyNumberTwo():Number{
       return MySecondNumber;
   }

}

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(
                                    "(new TestFunction(0,1,2,3,4,5)).length",
                                    6,
                                    (new TestFunction(0,1,2,3,4,5)).length );

    var myclass = new MyClass(10,20);

    array[item++] = Assert.expectEq(
                                    "new operator used to create objects",
                                    10,
                                    myclass.MyNumberOne() );

    array[item++] = Assert.expectEq(
                                    "new operator used to create objects",
                                    20,
                                    myclass.MyNumberTwo() );



    array[item++] = Assert.expectEq(
                                    "new operator used to create objects",
                                    "[object MyClass]",
                                    myclass+"" );

    return array;
}

function TestFunction() {
    return arguments;
}
