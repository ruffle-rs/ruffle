// Original source: https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/test/acceptance/Assert.as

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.adobe.test
{
import com.adobe.test.Utils;

/*
*	This Assert class is based off of the new ATS Assert class being implemented for AS4.
*	See: https://zerowing.corp.adobe.com/display/flashruntime/V12+ATS+APIs#V12ATSAPIs-AssertionActionScriptAPI
*  Also: https://zerowing.corp.adobe.com/display/flashruntime/V12+ATS+Specification
*/
public class Assert
{
  private static const PASSED = " PASSED!"
  private static const FAILED = " FAILED! expected: ";

  /**
   * Verifies that two values are equal. If not, an test failure is logged
   * for the test.
   * @param description	A unique description describing the test
   * that allows easy identification of the problem upon failure.
   * @param expected	The expected value of the actual argument.
   * @param actual	The value who's value is to be compared against
   * the value of expected.  This is done with strict equality (===).
   * @return True if the test passes, false if not. This can be used
   * to abort testing if you want fatal-style assertions.
   */
  public static function expectEq(description:String, expected:*, actual:*):Boolean
  {
    //  because ( NaN == NaN ) always returns false, need to do
    //  a special compare to see if we got the right result.
    if ( actual != actual ) {
      if ( typeof actual == "object" ) {
        actual = "NaN object";
      } else {
        actual = "NaN number";
      }
    }
    if ( expected != expected ) {
      if ( typeof expected == "object" ) {
        expected = "NaN object";
      } else {
        expected = "NaN number";
      }
    }
    var passed="";
    if (expected === actual) {
      if ( typeof(expected) != typeof(actual)  &&
        ! ( ( typeof(expected)=="float" && typeof(actual)=="number")
          || ( typeof(actual)=="float" && typeof(expected)=="number")
        )
      ){
        passed = "type error";
      } else {
        passed = "true";
      }
    } else { //expected != actual
      passed = "false";
      // if both objects are numbers
      // need to replace w/ IEEE standard for rounding
      if (typeof(actual) == "number" && typeof(expected) == "number") {
        if ( Math.abs(actual-expected) < 0.0000001 ) {
          passed = "true";
        }
      }

      // TODO: Float support
      /*
      // If both objects are float, check that the values are the same
      // within 7 digits of precision.
      // log_10(2^24) ie 24 bits gives 7.2 digits of decimal precision
      if (typeof(actual) == "float" && typeof(expected) == "float") {
          if ( float.abs(actual-expected) < float(0.000001) ) {
              passed = "true";
          }
      }
      */
    }

    printResult(description, expected, actual, passed);

    if (passed == "true")
      return true;
    else
      return false;
  }

  /**
   * Verifies that passed in function throws the expected Error.
   * This is NOT a supported ATS Assert, but used throughout vm testing.
   */
  // TODO: Should expected error be a string?
  public static function expectError(description:String, expectedError:String, testFunction:Function):Boolean
  {
    var actualErrorString:String = "No errors.";
    var actualError:* = null;
    try {
      testFunction();
    } catch (e:*) {
      actualErrorString = e.toString();
      actualError = e;
    }
    Utils.grabError(actualError, expectedError);
    return expectEq(description, expectedError, actualErrorString.substr(0, expectedError.length));
  }

  /**
   * Verifies that a value is true. If not, an test failure is logged for
   * the test.
   * @param description	A unique description describing the test
   * that allows easy identification of the problem upon failure.
   * @param actual	The value who's value is to be compared against
   * the value of true.  This is done with strict equality (===).
   * @return True if the test passes, false if not. This can be used
   * to abort testing if you want fatal-style assertions.
   */
  public function expectTrue(description:String, actual:*):Boolean
  {
    return expectEq(description, true, actual);
  }

  /**
   * Verifies that a value is false. If not, an test failure is logged for
   * the test.
   * @param description	A unique description describing the test
   * that allows easy identification of the problem upon failure.
   * @param actual	The value who's value is to be compared against
   * the value of false.  This is done with strict equality (===).
   * @return True if the test passes, false if not. This can be used
   * to abort testing if you want fatal-style assertions.
   */
  public function expectFalse(description:String, actual:*):Boolean
  {
    return expectEq(description, false, actual);
  }

  /**
   * Verifies that a value is null. If not, an test failure is logged for
   * the test.
   * @param description	A unique description describing the test
   * that allows easy identification of the problem upon failure.
   * @param actual	The value who's value is to be compared against
   * the value of null.  This is done with strict equality (===).
   * @return True if the test passes, false if not. This can be used
   * to abort testing if you want fatal-style assertions.
   */
  public function expectNull(description:String, actual:*):Boolean
  {
    return expectEq(description, null, actual);
  }

  /**
   * Verifies that a value is of a specific type, for example if a certain
   * error that is thrown is of the expected error type. If not, an test
   * failure is logged for the test.
   * @param description	A unique description describing the test
   * that allows easy identification of the problem upon failure.
   * @param expectedType	The expected type of the actual argument.
   * @param actual	The value who's type is to be compared against
   * the value of expected.  This is done with an 'is' comparison (is).
   * @return True if the test passes, false if not. This can be used
   * to abort testing if you want fatal-style assertions.
   */
  public function expectType(description:String, expectedType:*, actualObject:*):Boolean
  {
    var result:Boolean = (actualObject is expectedType);
    printResult(description, expectedType, actualObject, "type error");
    return result;
  }

  /**
   * Identifies a test as passing.
   */
  public function pass(description:String):void
  {
    print(description + PASSED);
  }

  /**
   * Identifies a test as failing.
   */
  public function fail(description:String):void
  {
    print(description + FAILED);
  }

  private static function printResult(description:String, expected:*, actual:*, result:String):void
  {
    if (result == "true") {
      description += PASSED;
    } else if (result == "false") {
      description += FAILED + expected + " got: " + actual;
    } else if (result == "type error") {
      description += FAILED + expected + " Type Mismatch - Expected Type: "+ typeof(expected) + ", Result Type: "+ typeof(actual);
    } else { //should never happen
      description += FAILED + " UNEXPECTED ERROR - see Assert.as:printResult()"
    }

    print(description);
  }
}
}
