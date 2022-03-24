package com.example.ruffle2;


import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;

import android.app.NativeActivity;
import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import android.view.View;

import java.io.IOException;
import java.io.InputStream;

public class MainActivity extends AppCompatActivity {

    static {
        System.loadLibrary("wgpu_android");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        ActivityResultLauncher launcher = registerForActivityResult(new ActivityResultContracts.GetContent(),
                uri -> {
                    /*try */{
                        Intent intent = new Intent(MainActivity.this, NativeActivity.class);

                        /*
                        InputStream stream = getContentResolver().openInputStream(uri);
                        // assuming the whole contents will be available at once
                        int size = stream.available();
                        byte[] bytes = new byte[size];
                        stream.read(bytes);

                        intent.putExtra("SWF_BYTES", bytes);
                        */

                        intent.putExtra("SWF_URI", uri);

                        startActivity(intent);
                    }/* catch (IOException e) {
                        Log.e("rfl", "IO Error when loading SWF");
                    }*/
                }
                );

        View button = findViewById(R.id.button);

        button.setOnClickListener((event) -> {
            launcher.launch("*/*" /* "application/x-shockwave-flash" */ );
        });

    }
}