/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing try block with multiple catch blocks, the  catch block with parameter of type SyntaxError catching the Syntax error.  Syntax Errors are parser errors and cannot be caught by the try and catch block.  They are thrown during compilation time.  This test is to test throw statement throwing error of type syntax error and whether the particular catch block handles it/ignores it or some other errors are caught by the catch block with parameter of type Syntax error ";  // Provide ECMA section title or a description
var BUGNUMBER = "";



thisError = "no error";
       
try {
     throw new SyntaxError();
    } catch(e:ReferenceError){
       thisError="This is Reference Error";
    }catch(e1:ArgumentError){
       thisError="This is Argument Error";
    }catch(e2:URIError){
       thisError="This is URI Error";
    }catch(e3:EvalError){
       thisError="This is Eval Error";
    }catch(e4:RangeError){
       thisError="This is Range Error";
    }catch(e4:TypeError){
       thisError="This is Type Error";
    }catch(e5:SecurityError){
       thisError="This is security Error";
    }catch(e6:DefinitionError){
       thisError="This is Definition Error";
    }catch(e7:UninitializedError){
       thisError="This is Uninitialized Error";
    }catch(e8:SyntaxError){
       thisError="This is Syntax Error";
    }catch(e9:VerifyError){
       thisError="This is Verify Error";
    }catch(e10:Error){
      thisError = e10.toString();
    }finally{
    Assert.expectEq( "Testing try block with throw statement", "This is Syntax Error"        ,thisError );
     }


              // displays results.
