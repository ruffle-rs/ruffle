package rs.ruffle;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;

import android.app.NativeActivity;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;

public class MainActivity extends AppCompatActivity {

    static {
        // load the native activity
        System.loadLibrary("ruffle_android");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        ActivityResultLauncher launcher = registerForActivityResult(new ActivityResultContracts.GetContent(),
                uri -> {
                    Intent intent = new Intent(MainActivity.this, NativeActivity.class);
                    intent.putExtra("SWF_URI", uri);
                    startActivity(intent);
                });

        View button = findViewById(R.id.button);

        button.setOnClickListener((event) -> {
            // should really be: "application/x-shockwave-flash"
            launcher.launch("*/*");
        });

    }
}