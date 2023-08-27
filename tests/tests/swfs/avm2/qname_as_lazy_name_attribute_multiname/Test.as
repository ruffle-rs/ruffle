package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

var xml =
    <alpha>
    <bravo attr1="value1" attr2="wrong">
        <charlie attr1="value2" attr2="wrong"/>
    </bravo>
</alpha>;

var q = new QName("attr1");
trace("xml..@[q]: " + xml..@[q]);
