/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 *
 *
 * Date:    15 July 2002
 * SUMMARY: Testing functions with double-byte names
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=58274
 *
 * Here is a sample of the problem:
 *
 *    js> function f\u02B1 () {}
 *
 *    js> f\u02B1.toSource();
 *    function f¦() {}
 *
 *    js> f\u02B1.toSource().toSource();
 *    (new String("function f\xB1() {}"))
 *
 *
 * See how the high-byte information (the 02) has been lost?
 * The same thing was happening with the toString() method:
 *
 *    js> f\u02B1.toString();
 *
 *    function f¦() {
 *    }
 *
 *    js> f\u02B1.toString().toSource();
 *    (new String("\nfunction f\xB1() {\n}\n"))
 *
 */
//-----------------------------------------------------------------------------
//     var SECTION = "";
//     var VERSION = "";


    var ERR = 'UNEXPECTED ERROR! \n';
    var ERR_MALFORMED_NAME = ERR + 'Could not find function name in: \n\n';
//     var TITLE   = "Testing functions with double-byte names";
//     var bug = 58274;


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var UBound = 0;
    
    var summary = 'Testing functions with double-byte names';
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];
    var sEval;
    var sName;

    //Commenting out as eval is not supported any more
    
    /*try{
       sEval = "function f\u02B2() {return 42;}";
       eval(sEval);
    }catch(e:Error){
       thisError=e.toString();
    }finally{
       status = 'In the try and catch block';
       actual = thisError;
       expect = "EvalError: eval is not supported";
       addThis();
    }*/
    function f\u02B2() {return 42};
    //eval(sEval);
    sName = getFunctionName(f\u02B2);


    // Test function call -
   //TO-DO: replacing inSection(1) with "function string1"
    //status = inSection(1);
    status = "function string1";

    actual = f\u02B2();
    expect = 42;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    //Expected results should be changed after bug 164523 is fixed
    // Test both characters of function name -
    //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 2";
    actual = sName.charAt(0);
    expect = 'F';
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);

  //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 3";
    actual = sName.charAt(1);
    expect = 'u';
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);



    function f\u02B2\u0AAA () {return 84;};
    //eval(sEval);
    sName = getFunctionName(f\u02B2\u0AAA);


    // Test function call -
   //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 4";
    actual = f\u02B2\u0AAA();
    expect = 84;
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    // Test all three characters of function name -
    //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 5";
    actual = sName.charAt(0);
    expect = 'F';
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 6";
    actual = sName.charAt(1);
    expect = 'u';
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    //TO-DO: Removing inSection() with "function string 1"
    //status = inSection(1);
    status = "function string 7";
    actual = sName.charAt(2);
    expect = 'n';
    //addThis();
    array[item++] = Assert.expectEq( status, expect, actual);

    return array;
}



/*
 * Goal: test that f.toString() contains the proper function name.
 *
 * Note, however, f.toString() is implementation-independent. For example,
 * it may begin with '\nfunction' instead of 'function'. Therefore we use
 * a regexp to make sure we extract the name properly.
 *
 * Here we assume that f has been defined by means of a function statement,
 * and not a function expression (where it wouldn't have to have a name).
 *
 * Rhino uses a Unicode representation for f.toString(); whereas
 * SpiderMonkey uses an ASCII representation, putting escape sequences
 * for non-ASCII characters. For example, if a function is called f\u02B1,
 * then in Rhino the toString() method will present a 2-character Unicode
 * string for its name, whereas SpiderMonkey will present a 7-character
 * ASCII string for its name: the string literal 'f\u02B1'.
 *
 * So we force the lexer to condense the string before using it.
 * This will give uniform results in Rhino and SpiderMonkey.
 */
function getFunctionName(f)
{
  var s = condenseStr(f.toString());
  var re = /\s*function\s+(\S+)\s*\(/;
  var arr = s.match(re);

  if (!(arr && arr[1]))
    return ERR_MALFORMED_NAME + s;
  return arr[1];
}


/*
 * This function is the opposite of functions like escape(), which take
 * Unicode characters and return escape sequences for them. Here, we force
 * the lexer to turn escape sequences back into single characters.
 *
 * Note we can't simply do |eval(str)|, since in practice |str| will be an
 * identifier somewhere in the program (e.g. a function name); thus |eval(str)|
 * would return the object that the identifier represents: not what we want.
 *
 * So we surround |str| lexicographically with quotes to force the lexer to
 * evaluate it as a string. Have to strip out any linefeeds first, however -
 */
function condenseStr(str)
{
  /*
   * You won't be able to do the next step if |str| has
   * any carriage returns or linefeeds in it. For example:
   *
   *  js> eval("'" + '\nHello' + "'");
   *  1: SyntaxError: unterminated string literal:
   *  1: '
   *  1: ^
   *
   * So replace them with the empty string -
   */
  str = str.replace(/[\r\n]/g, '')
  
  //return eval("'" + str + "'");
  return ("'"+ str + "'");
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

/*
function test()
{
  enterFunc('test');
  printBugNumber(bug);
  printStatus(summary);

  for (var i=0; i<UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
  }

  exitFunc ('test');
}
 */
