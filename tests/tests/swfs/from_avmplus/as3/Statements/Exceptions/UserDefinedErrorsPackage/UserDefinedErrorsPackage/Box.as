/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package UserDefinedErrorsPackage
{
class Box
{
  private var width:Number;

  public function setWidth(w):Boolean
  {
    var errmsg:String="Illegal Box Dimension specified";
    var errmsg2:String="Box dimensions should be greater than 0";
    var errmsg3:String="Box dimensions must be less than Number.MAX_VALUE";
    if (w == NaN)
    {
      throw new BoxDimensionException(errmsg);
    }else if (w<= 0)
    {
      throw new BoxUnderzeroException(errmsg2);
    }else if (w>Number.MAX_VALUE)
    {
      throw new BoxOverflowException(errmsg3);
    }
    width = w;

  }
}
}
