/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package UserDefinedErrorsInPackage
{
    import com.adobe.test.Assert;
     public class TryAndCatchBlockWithUserDefinedErrorsInPackage
     {
         var b:Box = new Box();
         var someWidth:Number=-10;
         thisError = "no error";

         public function MyTryThrowCatchFunction():void
         {
             try {
                 b.setWidth(someWidth);
                 }catch(e:BoxOverflowException){
                     thisError = e.message;
                     //print(thisError);
                     //trace("BoxOverflowException:"+thisError);
                 /*}catch(e1:BoxUnderzeroException){
                     thisError=e1.message;
                     //print(thisError);*/
                     //trace("BoxUnderzeroException:"+thisError);
                 }catch(e2:BoxDimensionException){
                     thisError = e2.message;
                     //print(thisError);
                     //trace("BoxDimensionException Occurred:"+thisError());
                 }catch(e3:Error){
                     thisError=e3.message;
                     //print(e3.toString());
                     //trace("An error occurred:"+e3.toString());
                 }finally{
                      Assert.expectEq( "Testing try block and multiple catch blocks with custom error classes", "Box dimensions should be greater than 0",thisError );
                  }
          }
      }
}

