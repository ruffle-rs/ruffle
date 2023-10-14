/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_4_4";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    //  special case:  var is not defined
    var MYVAR;
    array[item++] = Assert.expectEq(   "var MYVAR; ++MYVAR",                       NaN,                             ++MYVAR );
    var MYVAR=void 0;
    array[item++] = Assert.expectEq(   "var MYVAR= void 0; ++MYVAR",               NaN,                             ++MYVAR );
    var MYVAR=null;
    array[item++] = Assert.expectEq(   "var MYVAR=null; ++MYVAR",                  1,                             ++MYVAR );
    var MYVAR=true;
    array[item++] = Assert.expectEq(   "var MYVAR=true; ++MYVAR",                  2,                             ++MYVAR );
    var MYVAR=false;
    array[item++] = Assert.expectEq(   "var MYVAR=false; ++MYVAR",                 1,                            ++MYVAR );

    // special numbers
    // verify return value
     var MYVAR=Number.POSITIVE_INFINITY;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.POSITIVE_INFINITY;++MYVAR", Number.POSITIVE_INFINITY,  ++MYVAR );
    var MYVAR=Number.NEGATIVE_INFINITY;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NEGATIVE_INFINITY;++MYVAR", Number.NEGATIVE_INFINITY,   ++MYVAR );
     var MYVAR=Number.NaN;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NaN;++MYVAR",               Number.NaN,                ++MYVAR );

    // verify value of variable
    var MYVAR=Number.POSITIVE_INFINITY;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.POSITIVE_INFINITY;++MYVAR;MYVAR", Number.POSITIVE_INFINITY,   MYVAR );
    var MYVAR=Number.NEGATIVE_INFINITY;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NEGATIVE_INFINITY;++MYVAR;MYVAR", Number.NEGATIVE_INFINITY,   MYVAR );
    var MYVAR=Number.NaN;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NaN;++MYVAR;MYVAR",               Number.NaN,                 MYVAR );


    // number primitives
    var MYVAR=0;
    array[item++] = Assert.expectEq(     "var MYVAR=0;++MYVAR",            1,          ++MYVAR );
    var MYVAR=0.2345;
    array[item++] = Assert.expectEq(     "var MYVAR=0.2345;++MYVAR",       1.2345,     ++MYVAR );
    var MYVAR=-0.2345;
    array[item++] = Assert.expectEq(     "var MYVAR=-0.2345;++MYVAR",      0.7655000000000001,     ++MYVAR );

    // verify value of variable
    var MYVAR=0;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=0;++MYVAR;MYVAR",      1,         MYVAR );
    var MYVAR=0.2345;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=0.2345;++MYVAR;MYVAR", 1.2345,    MYVAR );
    var MYVAR=-0.2345;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=-0.2345;++MYVAR;MYVAR", 0.7655000000000001,   MYVAR );
    var MYVAR=0;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=0;++MYVAR;MYVAR",      1,   MYVAR );
    var MYVAR=0;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=0;++MYVAR;MYVAR",      1,   MYVAR );
    var MYVAR=0;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=0;++MYVAR;MYVAR",      1,   MYVAR );

    // boolean values
    // verify return value
    var MYVAR=true;
    array[item++] = Assert.expectEq(     "var MYVAR=true;++MYVAR",         2,       ++MYVAR );
    var MYVAR=false;
    array[item++] = Assert.expectEq(     "var MYVAR=false;++MYVAR",        1,      ++MYVAR );
    // verify value of variable
    var MYVAR=true;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=true;++MYVAR;MYVAR",   2,   MYVAR );
    var MYVAR=false;++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=false;++MYVAR;MYVAR",  1,   MYVAR );

    // boolean objects
    // verify return value
    var MYVAR=true;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(true);++MYVAR",         2,     ++MYVAR );
    var MYVAR=false;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(false);++MYVAR",        1,     ++MYVAR );
    // verify value of variable
    var MYVAR=new Boolean(true);++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(true);++MYVAR;MYVAR",   2,     MYVAR );
    var MYVAR=new Boolean(false);++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(false);++MYVAR;MYVAR",  1,     MYVAR );

    // string primitives
    var MYVAR='string';
    array[item++] = Assert.expectEq(     "var MYVAR='string';++MYVAR",         Number.NaN,     ++MYVAR );
    var MYVAR='12345';
    array[item++] = Assert.expectEq(     "var MYVAR='12345';++MYVAR",          12346,          ++MYVAR );
    var MYVAR='-12345';
    array[item++] = Assert.expectEq(     "var MYVAR='-12345';++MYVAR",         -12344,         ++MYVAR );
    var MYVAR='0Xf';
    array[item++] = Assert.expectEq(     "var MYVAR='0Xf';++MYVAR",            16,             ++MYVAR );
    var MYVAR='077';
    array[item++] = Assert.expectEq(     "var MYVAR='077';++MYVAR",            78,             ++MYVAR );
    var MYVAR='';
    array[item++] = Assert.expectEq(     "var MYVAR=''; ++MYVAR",              1,              ++MYVAR );

    // verify value of variable
    var MYVAR='string';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='string';++MYVAR;MYVAR",   Number.NaN,     MYVAR );
    var MYVAR='12345';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='12345';++MYVAR;MYVAR",    12346,          MYVAR );
     var MYVAR='-12345';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='-12345';++MYVAR;MYVAR",   -12344,         MYVAR );
    var MYVAR='0xf';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='0xf';++MYVAR;MYVAR",      16,             MYVAR );
    var MYVAR='077';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='077';++MYVAR;MYVAR",      78,             MYVAR );
    var MYVAR='';++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR='';++MYVAR;MYVAR",         1,              MYVAR );

    // string objects
    var MYVAR=new String('string');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('string');++MYVAR",         Number.NaN,     ++MYVAR );
    var MYVAR=new String('12345');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('12345');++MYVAR",          12346,          ++MYVAR );
    var MYVAR=new String('-12345');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('-12345');++MYVAR",         -12344,         ++MYVAR );
    var MYVAR=new String('0Xf');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('0Xf');++MYVAR",            16,             ++MYVAR );
    var MYVAR=new String('077');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('077');++MYVAR",            78,             ++MYVAR );
    var MYVAR=new String('');
    array[item++] = Assert.expectEq(     "var MYVAR=new String(''); ++MYVAR",              1,              ++MYVAR );

    // verify value of variable
    var MYVAR=new String('string');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('string');++MYVAR;MYVAR",   Number.NaN,     MYVAR );
    var MYVAR=new String('12345');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('12345');++MYVAR;MYVAR",    12346,          MYVAR );
    var MYVAR=new String('-12345');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('-12345');++MYVAR;MYVAR",   -12344,          MYVAR );
    var MYVAR=new String('0xf');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('0xf');++MYVAR;MYVAR",      16,             MYVAR );
    var MYVAR=new String('077');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('077');++MYVAR;MYVAR",      78,             MYVAR );
    var MYVAR=new String('');++MYVAR;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('');++MYVAR;MYVAR",         1,              MYVAR );

    return ( array );
}
