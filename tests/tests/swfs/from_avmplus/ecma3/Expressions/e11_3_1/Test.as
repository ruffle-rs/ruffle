/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "11_3_1";
//     var VERSION = "ECMA_1";


    testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    // special numbers
    var MYVAR;
    array[item++] = Assert.expectEq(   "var MYVAR; MYVAR++",                       NaN,                             MYVAR++ );
    var MYVAR=void 0;
    array[item++] = Assert.expectEq(   "var MYVAR= void 0; MYVAR++",               NaN,                             MYVAR++ );
    var MYVAR=null;
    array[item++] = Assert.expectEq(   "var MYVAR=null; MYVAR++",                  0,                            MYVAR++ );
      var MYVAR=true;
    array[item++] = Assert.expectEq(   "var MYVAR=true; MYVAR++",                  1,                          MYVAR++ );
    var MYVAR=false;
    array[item++] = Assert.expectEq(   "var MYVAR=false; MYVAR++",                 0,                            MYVAR++ );

    // verify return value
    var MYVAR=Number.POSITIVE_INFINITY;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.POSITIVE_INFINITY;MYVAR++", Number.POSITIVE_INFINITY,   MYVAR++ );
    var MYVAR=Number.NEGATIVE_INFINITY;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NEGATIVE_INFINITY;MYVAR++", Number.NEGATIVE_INFINITY,   MYVAR++ );
    var MYVAR=Number.NaN;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NaN;MYVAR++",               Number.NaN,                 MYVAR++ );

    // verify value of variable
    var MYVAR=Number.POSITIVE_INFINITY;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.POSITIVE_INFINITY;MYVAR++;MYVAR", Number.POSITIVE_INFINITY,   MYVAR );
    var MYVAR=Number.NEGATIVE_INFINITY;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NEGATIVE_INFINITY;MYVAR++;MYVAR", Number.NEGATIVE_INFINITY,   MYVAR );
    var MYVAR=Number.NaN;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=Number.NaN;MYVAR++;MYVAR",               Number.NaN,                 MYVAR );

    // number primitives
      var MYVAR=0;
    array[item++] = Assert.expectEq(     "var MYVAR=0;MYVAR++",            0,        MYVAR++ );
    var MYVAR=0.2345;
    array[item++] = Assert.expectEq(     "var MYVAR=0.2345;MYVAR++",       0.2345,     MYVAR++ );
      var MYVAR=-0.2345;
    array[item++] = Assert.expectEq(     "var MYVAR=-0.2345;MYVAR++",      -0.2345,   MYVAR++ );

    // verify value of variable
    var MYVAR=0;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=0;MYVAR++;MYVAR",      1,          MYVAR );
    var MYVAR=0.2345;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=0.2345;MYVAR++;MYVAR", 1.2345,     MYVAR );
    var MYVAR=-0.2345;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=-0.2345;MYVAR++;MYVAR", 0.7655000000000001,   MYVAR );
    var MYVAR=0;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=0;MYVAR++;MYVAR",      1,   MYVAR );
    var MYVAR=0;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=0;MYVAR++;MYVAR",      1,   MYVAR );
    var MYVAR=0;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=0;MYVAR++;MYVAR",      1,   MYVAR );

    // boolean values
    // verify return value
    var MYVAR=true;
    array[item++] = Assert.expectEq(     "var MYVAR=true;MYVAR++",         1,       MYVAR++ );
    var MYVAR=false;
    array[item++] = Assert.expectEq(     "var MYVAR=false;MYVAR++",        0,      MYVAR++ );
    // verify value of variable
    var MYVAR=true;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=true;MYVAR++;MYVAR",   2,   MYVAR );
    var MYVAR=false;MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=false;MYVAR++;MYVAR",  1,   MYVAR );

    // boolean objects
    // verify return value
    var MYVAR=true;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(true);MYVAR++",         1,     MYVAR++ );
    var MYVAR=false;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(false);MYVAR++",        0,     MYVAR++ );
    // verify value of variable
    var MYVAR=new Boolean(true);MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(true);MYVAR++;MYVAR",   2,     MYVAR );
    var MYVAR=new Boolean(false);MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new Boolean(false);MYVAR++;MYVAR",  1,     MYVAR );

    // string primitives

    var MYVAR = 'string';
    array[item++] = Assert.expectEq(     "var MYVAR='string'; MYVAR++",         Number.NaN,     MYVAR++ );
    var MYVAR = '12345';
    array[item++] = Assert.expectEq(     "var MYVAR='12345';MYVAR++",          12345,          MYVAR++ );
    var MYVAR='-12345';
    array[item++] = Assert.expectEq(     "var MYVAR='-12345';MYVAR++",         -12345,         MYVAR++ );
    var MYVAR='0Xf';
    array[item++] = Assert.expectEq(     "var MYVAR='0Xf';MYVAR++",            15,             MYVAR++ );
    var MYVAR='077';
    array[item++] = Assert.expectEq(     "var MYVAR='077';MYVAR++",            77,             MYVAR++ );
     var MYVAR='';
    array[item++] = Assert.expectEq(     "var MYVAR=''; MYVAR++",              0,             MYVAR++ );

    // verify value of variable
    var MYVAR='string';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='string';MYVAR++;MYVAR",   Number.NaN,     MYVAR );
     var MYVAR='12345';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='12345';MYVAR++;MYVAR",    12346,         MYVAR );
    var MYVAR='-12345';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='-12345';MYVAR++;MYVAR",   -12344,          MYVAR );
    var MYVAR='0xf';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='0xf';MYVAR++;MYVAR",      16,             MYVAR );
    var MYVAR='077';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='077';MYVAR++;MYVAR",      78,             MYVAR );
    var MYVAR='';MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR='';MYVAR++;MYVAR",         1,              MYVAR );

    // string objects
    var MYVAR=new String('string');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('string');MYVAR++",         Number.NaN,     MYVAR++ );
     var MYVAR=new String('12345');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('12345');MYVAR++",          12345,         MYVAR++ );
     var MYVAR=new String('-12345');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('-12345');MYVAR++",         -12345,        MYVAR++ );
    var MYVAR=new String('0Xf');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('0Xf');MYVAR++",            15,             MYVAR++ );
    var MYVAR=new String('077');
    array[item++] = Assert.expectEq(     "var MYVAR=new String('077');MYVAR++",            77,             MYVAR++ );
    var MYVAR=new String('');
    array[item++] = Assert.expectEq(     "var MYVAR=new String(''); MYVAR++",              0,              MYVAR++ );

    // verify value of variable
    var MYVAR=new String('string');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('string');MYVAR++;MYVAR",   Number.NaN,     MYVAR );
    var MYVAR=new String('12345');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('12345');MYVAR++;MYVAR",    12346,          MYVAR );
     var MYVAR=new String('-12345');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('-12345');MYVAR++;MYVAR",   -12344,         MYVAR );
    var MYVAR=new String('0xf');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('0xf');MYVAR++;MYVAR",      16,             MYVAR );
    var MYVAR=new String('077');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('077');MYVAR++;MYVAR",      78,             MYVAR );
    var MYVAR=new String('');MYVAR++;
    array[item++] = Assert.expectEq(     "var MYVAR=new String('');MYVAR++;MYVAR",         1,              MYVAR );

    // array elements
    var MYVAR = ["string", null, undefined, 300];
    MYVAR[0]++;
    array[item++] = Assert.expectEq(     "increment a string in array", Number.NaN, MYVAR[0]);
    
    MYVAR[1]++;
    array[item++] = Assert.expectEq(     "increment a null in array", 1, MYVAR[1]);
    
    MYVAR[2]++;
    array[item++] = Assert.expectEq(     "increment a undefined in array", Number.NaN, MYVAR[2]);
    
    MYVAR[3]++;
    array[item++] = Assert.expectEq(     "increment a number in array", 301, MYVAR[3]);
    
    // object
    var o = {"num":22};
    o.x++;
    array[item++] = Assert.expectEq(     "increment a non-existant object property", Number.NaN, o.x);
    o.num++;
    array[item++] = Assert.expectEq(     "increment an object property", 23, o.num);
    

    return ( array );
}
