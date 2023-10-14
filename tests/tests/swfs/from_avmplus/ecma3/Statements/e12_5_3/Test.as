/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
  

//     var SECTION = "12.5";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The if statment";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var MYVAR;
    if ( true )
        MYVAR='PASSED';
    else if(false)
        MYVAR= 'FAILED';
    array[item++] = Assert.expectEq(   
                                    "var MYVAR; if ( true ) MYVAR='PASSED'; else if (false) MYVAR= 'FAILED';",
                                    "PASSED",
                                     MYVAR);
    var MYVAR;
    if ( false )
        MYVAR='FAILED';
    else if (true)
        MYVAR= 'PASSED';
    array[item++] = Assert.expectEq(  
                                    "var MYVAR; if ( false ) MYVAR='FAILED'; else if (true) MYVAR= 'PASSED';",
                                    "PASSED",
                                     MYVAR);
    var MYVAR;
    if ( new Boolean(true) )
        MYVAR='PASSED';
    else if (new Boolean(false))
        MYVAR= 'FAILED';
    array[item++] = Assert.expectEq(   
                                    "var MYVAR; if ( new Boolean(true) )  MYVAR='PASSED'; else if (new Boolean(false))MYVAR= 'FAILED';",
                                    "PASSED",
                                     MYVAR);
    var MYVAR;
    if ( new Boolean(false) )
        MYVAR='PASSED';
    else if (new Boolean(true))
        MYVAR= 'FAILED';
    array[item++] = Assert.expectEq(  
                                    "var MYVAR; if ( new Boolean(false) ) MYVAR='PASSED'; else if (new Boolean(true)) MYVAR= 'FAILED';",
                                    "FAILED",
                                    MYVAR);
    var MYVAR;
    if ( 1 )
        MYVAR='PASSED';
    else if (0)
        MYVAR= 'FAILED';
    array[item++] = Assert.expectEq(   
                                    "var MYVAR; if ( 1 ) MYVAR='PASSED'; else if (0) MYVAR= 'FAILED';",
                                    "PASSED",
                                    MYVAR);
    var MYVAR;
    if ( 0 )
        MYVAR='FAILED';
    else if (1)
        MYVAR= 'PASSED';
    array[item++] = Assert.expectEq(  
                                    "var MYVAR; if ( 0 ) MYVAR='FAILED'; else if (1) MYVAR= 'PASSED';","PASSED",MYVAR);

    var MyVar1 = 50;
    var MyVar2 = 100;

    if (MyVar1>MyVar2)
        result="MyVar1 is greater than MyVar2";
    else if (MyVar2==MyVar1)
        result="MyVar2 equals MyVar1";
    else
        result="MyVar2 greater than MyVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyVar2 greater than MyVar1",result);

    var MyVar1 = 100;
    var MyVar2 = 50;

    if (MyVar1>MyVar2)
        result="MyVar1 is greater than MyVar2";
    else if (MyVar2==MyVar1)
        result="MyVar2 equals MyVar1";
    else
        result="MyVar2 greater than MyVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyVar1 is greater than MyVar2",result);

    var MyVar1 = 50;
    var MyVar2 = 50;

    if (MyVar1>MyVar2)
        result="MyVar1 is greater than MyVar2";
    else if (MyVar2==MyVar1)
        result="MyVar2 equals MyVar1";
    else
        result="MyVar2 greater than MyVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyVar2 equals MyVar1",result);


    var MyStringVar1 = "string"
    var MyStringVar2 = "string";

    if (MyStringVar1>MyStringVar2)
        result="MyStringVar1 is greater than MyStringVar2";
    else if (MyStringVar2==MyStringVar1)
        result="MyStringVar2 equals MyStringVar1";
    else
        result="MyStringVar2 greater than MyStringVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyStringVar2 equals MyStringVar1",result);

    var MyStringVar1 = "String";
    var MyStringVar2 = "string";

    if (MyStringVar1>MyStringVar2)
        result="MyStringVar1 is greater than MyStringVar2";
    else if (MyStringVar2==MyStringVar1)
        result="MyStringVar2 equals MyStringVar1";
    else
        result="MyStringVar2 greater than MyStringVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyStringVar2 greater than MyStringVar1",result);


    var MyStringVar1 = "strings";
    var MyStringVar2 = "string";

    if (MyStringVar1>MyStringVar2)
        result="MyStringVar1 is greater than MyStringVar2";
    else if (MyStringVar2==MyStringVar1)
        result="MyStringVar2 equals MyStringVar1";
    else
        result="MyStringVar2 greater than MyStringVar1";

    array[item++] = Assert.expectEq(  "Testing if elseif else","MyStringVar1 is greater than MyStringVar2",result);

    return array;
}
