/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "11.8.7";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The in operator";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
  
    var Array_One = new Array(0,1,2,3);

    

    array[item++] = Assert.expectEq(   
                                    "0 in Array_One",
                                    true,
                                    0 in Array_One);

    array[item++] = Assert.expectEq(   
                                    "1 in Array_One",
                                    true,
                                    1 in Array_One);

    array[item++] = Assert.expectEq(   
                                    "2 in Array_One",
                                    true,
                                    2 in Array_One);

    array[item++] = Assert.expectEq(   
                                    "3 in Array_One",
                                    true,
                                    3 in Array_One);



    array[item++] = Assert.expectEq(   
                                    "4 in Array_One",
                                    false,
                                    4 in Array_One);

    var Array_Two = new Array('z','y','x','w');

     array[item++] = Assert.expectEq(   
                                    "0 in Array_Two",
                                    true,
                                    0 in Array_Two);

    array[item++] = Assert.expectEq(   
                                    "1 in Array_Two",
                                    true,
                                    1 in Array_Two);

    array[item++] = Assert.expectEq(   
                                    "2 in Array_Two",
                                    true,
                                    2 in Array_Two);

    array[item++] = Assert.expectEq(   
                                    "3 in Array_Two",
                                    true,
                                    3 in Array_Two);



    array[item++] = Assert.expectEq(   
                                    "4 in Array_Two",
                                    false,
                                    4 in Array_Two);

    array[item++] = Assert.expectEq(   
                                    "a in Array_Two",
                                    false,
                                    'a' in Array_Two);

    array[item++] = Assert.expectEq(   
                                    "length in Array_Two",
                                    true,
                                    "length" in Array_Two);

    var myobj = {obj1:"string1",obj2:"string2"}

    array[item++] = Assert.expectEq(   
                                    "obj1 in myobj",
                                    true,
                                    "obj1" in myobj);

    array[item++] = Assert.expectEq(   
                                    "obj2 in myobj",
                                    true,
                                    "obj2" in myobj);

    function myfunc():String{
        return "Hi!!!"}

    MyObject2 = {MyNumber:10,MyString:'string',MyBoolean:true,myarr:[1,2,3],myfuncvar:myfunc}

    array[item++] = Assert.expectEq(   
                                    "MyNumber in MyObject2",
                                    true,
                                    "MyNumber" in MyObject2);

     array[item++] = Assert.expectEq(   
                                    "MyString in MyObject2",
                                    true,
                                    "MyString" in MyObject2);

     array[item++] = Assert.expectEq(   
                                    "MyBoolean in MyObject2",
                                    true,
                                    "MyBoolean" in MyObject2);



     array[item++] = Assert.expectEq(   
                                    "myarr in MyObject2",
                                    true,
                                    "myarr" in MyObject2);

     array[item++] = Assert.expectEq(   
                                    "myfuncvar in MyObject2",
                                    true,
                                    "myfuncvar" in MyObject2);


     var mystring1 = new String("string1");

     array[item++] = Assert.expectEq(   
                                    "length in mystring1",
                                    true,
                                    "length" in mystring1);

     var mystring2 = "string2";

      
      array[item++] = Assert.expectEq(   
                                    "length in mystring2",
                                    true,
                                    "length" in mystring2);

     MyObject3 = {MyNumber1:10,MyString1:'string',MyBoolean1:true,myarr1:[1,2,3],myfuncvar1:myfunc}

     delete MyObject3.MyNumber1;


    array[item++] = Assert.expectEq(   
                                    "MyNumber1 in MyObject3",
                                    false,
                                    "MyNumber1" in MyObject3);
    

     delete MyObject3.myfuncvar1;

    array[item++] = Assert.expectEq(   
                                    "myfuncvar1 in MyObject3",
                                    false,
                                    "myfuncvar1" in MyObject3);

    MyObject3.MyNumber1 = undefined;

    array[item++] = Assert.expectEq(   
                                    "MyNumber1 in MyObject3",
                                    true,
                                    "MyNumber1" in MyObject3);

    myarr3 = [0,1,2,3];

    delete myarr3[3];

    array[item++] = Assert.expectEq(   
                                    "3 in myarr3",
                                    false,
                                    3 in myarr3);

    myarr3[3] = undefined;

    array[item++] = Assert.expectEq(   
                                    "3 in myarr3",
                                    true,
                                    3 in myarr3);



    array[item++] = Assert.expectEq(   
                                    "PI in Math",
                                    true,
                                    "PI" in Math);

    array[item++] = Assert.expectEq(   
                                    "myproperty in Math",
                                    false,
                                    "myproperty" in Math);

    array[item++] = Assert.expectEq(   
                                    "myproperty in Object",
                                    false,
                                    "myproperty" in Object)


    

    
  
    return ( array );
}

