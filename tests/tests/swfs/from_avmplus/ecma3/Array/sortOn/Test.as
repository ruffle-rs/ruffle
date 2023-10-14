/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *  File Name:          sortOn.as
 *  ECMA Section:       15.4.4.3 Array.sortOn()
 *  Description:        Test Case for sortOn function of Array Class.
 *          Here three objects have been created and using the
 *          sortOn() function, we are sorting the fields in the
 *          Object.
 *
 *  Author:         Gagneet Singh (gasingh@macromedia.com)
 *  Date:           01/09/2005
 *  Modified By:        Subha Subramanian
 *  Date:               01/05/2006
 *  Details:            Added more tests to test sortOn method and added tests to test        *  sortOn method's  changed sorting behavior with Array class constants
 */

// var SECTION = "sortOn";
// var TITLE   = "Array.sortOn";

// var VERSION = "ECMA_3";




var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    // Create three objects to hold
    var OBJ1    = new Object();
    OBJ1.name   = "Chris";
    OBJ1.city   = "Dallas";
    OBJ1.zip    = 72501;

    var OBJ2    = new Object();
    OBJ2.name   = "Mike";
    OBJ2.city   = "Newton";
    OBJ2.zip    = 68144;

    var OBJ3    = new Object();
    OBJ3.name   = "Greg";
    OBJ3.city   = "San Francisco";
    OBJ3.zip    = 94103;


    // Create an array to hold the three objects contents in an array object.
    var MYARRAY = new Array();

    // Push the objects created into the array created.
    MYARRAY.push(OBJ1);
    MYARRAY.push(OBJ2);
    MYARRAY.push(OBJ3);

    // Array to hold the output string as they were before.
    var EXPECT_VAR = new Array();

    // Output the current state of the array
    for (var SORTVAR = 0; ( SORTVAR < MYARRAY.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (MYARRAY[SORTVAR].city);
    }


    // Sort the array on the City field.
    MYARRAY.sortOn("city");


    // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < MYARRAY.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR.sortOn(city)",  EXPECT_VAR[MYVAR],  MYARRAY[MYVAR].city );
    }
        var RESULT_ARRAY = new Array();
        for (var SORTVAR = 0; ( SORTVAR < MYARRAY.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (MYARRAY[SORTVAR].name);

    }
        RESULT_ARRAY = EXPECT_VAR.sort();
        // Sort the array on the name field.
    MYARRAY.sortOn("name");


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < MYARRAY.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR.sortOn(name)",  RESULT_ARRAY[MYVAR],  MYARRAY[MYVAR].name );
    }

        for (var SORTVAR = 0; ( SORTVAR < MYARRAY.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (MYARRAY[SORTVAR].zip);

    }
        RESULT_ARRAY = EXPECT_VAR.sort();
        // Sort the array on the zip field.
    MYARRAY.sortOn("zip");


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < MYARRAY.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR.sortOn(zip)",  RESULT_ARRAY[MYVAR],  MYARRAY[MYVAR].zip );
    }

        //Using constants to change the sorting behavior



        for (var SORTVAR = 0; ( SORTVAR < MYARRAY.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (MYARRAY[SORTVAR].zip);

    }
        RESULT_ARRAY = EXPECT_VAR.sort(Array.NUMERIC);
        // Sort the array on the name field.
    MYARRAY.sortOn("zip",Array.NUMERIC);


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < MYARRAY.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "MYARR.sortOn(zip,Array.NUMERIC)",  RESULT_ARRAY[MYVAR],  MYARRAY[MYVAR].zip );
    }


        var users:Array = [{name:"Bob",age:3},{name:"barb",age:35},{name:"abcd",age:3},{name:"catchy",age:4}]

        for (var SORTVAR = 0; ( SORTVAR < users.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (users[SORTVAR].name);

    }
        RESULT_ARRAY = EXPECT_VAR.sort(Array.CASEINSENSITIVE);
        // Sort the array on the name field.
    users.sortOn("name",Array.CASEINSENSITIVE);


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < users.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "users.sortOn(name,Array.CASEINSENSITIVE)",  RESULT_ARRAY[MYVAR],  users[MYVAR].name );
    }
        RESULT_ARRAY = EXPECT_VAR.sort(Array.CASEINSENSITIVE|Array.DESCENDING);
        // Sort the array on the name field.
    users.sortOn("name",Array.CASEINSENSITIVE|Array.DESCENDING);


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < users.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "users.sortOn(name,Array.CASEINSENSITIVE|Array.DESCENDING)",  RESULT_ARRAY[MYVAR],  users[MYVAR].name );
    }

        for (var SORTVAR = 0; ( SORTVAR < users.length ); SORTVAR++)
    {
        EXPECT_VAR[SORTVAR] = (users[SORTVAR].age);

    }
        RESULT_ARRAY = EXPECT_VAR.sort();
        // Sort the array on the age field.
    users.sortOn("age");


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < users.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "users.sortOn(age)",  RESULT_ARRAY[MYVAR],  users[MYVAR].age );
    }

        RESULT_ARRAY = EXPECT_VAR.sort(Array.NUMERIC);
        // Sort the array on the age field.
    users.sortOn("age",Array.NUMERIC);


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < users.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "users.sortOn(age,Array.NUMERIC)",  RESULT_ARRAY[MYVAR],  users[MYVAR].age );
    }

        RESULT_ARRAY = EXPECT_VAR.sort(Array.DESCENDING|Array.NUMERIC);
        // Sort the array on the age field.
    users.sortOn("age",Array.DESCENDING|Array.NUMERIC);


        // Output the result of the Sort Operation.
    for (var MYVAR = 0; ( MYVAR < users.length ); MYVAR++)
    {
        array[item++] = Assert.expectEq(  "users.sortOn(age,Array.DESCENDING|Array.NUMERIC)",  RESULT_ARRAY[MYVAR],  users[MYVAR].age );
    }



    return ( array );

}
