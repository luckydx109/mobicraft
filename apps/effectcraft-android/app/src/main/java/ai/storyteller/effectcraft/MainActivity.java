package ai.storyteller.effectcraft;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.ContentResolver;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Build;
import android.provider.MediaStore;
import android.provider.OpenableColumns;
import android.util.Log;
import android.webkit.MimeTypeMap;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

/**
 * The app's NativeActivity plus the two system pickers the import screen opens: the gallery
 * (photos and videos) and the file manager (audio).
 *
 * The Rust side calls {@link #pickMedia(String)}. What the user picks is copied into
 * {@code <app files>/imports} (the file manager and the gallery hand out content:// URIs the engine
 * cannot open), and the copied paths are written to {@code imports/picked.txt}, one per line. An
 * empty file means the picker was cancelled. Rust polls that file (storage_access.rs).
 */
public class MainActivity extends NativeActivity {
    private static final String TAG = "EffectCraft";
    private static final int REQ_MEDIA = 4712;
    private static final int REQ_AUDIO = 4713;

    /** Called from native code on any thread. {@code kind}: "media" (gallery) or "audio" (files). */
    public void pickMedia(final String kind) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                launch(kind);
            }
        });
    }

    private void launch(String kind) {
        try {
            Intent intent;
            int request;
            if ("audio".equals(kind)) {
                intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("audio/*");
                intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);
                request = REQ_AUDIO;
            } else if (Build.VERSION.SDK_INT >= 33) {
                // The system photo picker: the gallery, photos and videos, several at once.
                intent = new Intent(MediaStore.ACTION_PICK_IMAGES);
                intent.putExtra(MediaStore.EXTRA_PICK_IMAGES_MAX, MediaStore.getPickImagesMaxLimit());
                request = REQ_MEDIA;
            } else {
                intent = new Intent(Intent.ACTION_GET_CONTENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                intent.putExtra(Intent.EXTRA_MIME_TYPES, new String[] {"image/*", "video/*"});
                intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);
                request = REQ_MEDIA;
            }
            startActivityForResult(intent, request);
        } catch (Exception e) {
            Log.e(TAG, "picker could not open: " + e);
            writeResult(new ArrayList<String>());
        }
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != REQ_MEDIA && request != REQ_AUDIO) {
            return;
        }
        final List<Uri> uris = new ArrayList<Uri>();
        if (result == RESULT_OK && data != null) {
            ClipData clip = data.getClipData();
            if (clip != null) {
                for (int i = 0; i < clip.getItemCount(); i++) {
                    Uri u = clip.getItemAt(i).getUri();
                    if (u != null) {
                        uris.add(u);
                    }
                }
            } else if (data.getData() != null) {
                uris.add(data.getData());
            }
        }
        // Copying a video can take a while: not on the UI thread.
        new Thread(new Runnable() {
            @Override
            public void run() {
                copyAll(uris);
            }
        }).start();
    }

    private void copyAll(List<Uri> uris) {
        List<String> out = new ArrayList<String>();
        File dir = importsDir();
        for (Uri uri : uris) {
            try {
                File f = copy(uri, dir);
                if (f != null) {
                    out.add(f.getAbsolutePath());
                }
            } catch (Exception e) {
                Log.e(TAG, "could not copy " + uri + ": " + e);
            }
        }
        writeResult(out);
    }

    /** The same folder the Rust side reads: getExternalFilesDir(null), else getFilesDir(). */
    private File importsDir() {
        File base = getExternalFilesDir(null);
        if (base == null) {
            base = getFilesDir();
        }
        File dir = new File(base, "imports");
        dir.mkdirs();
        return dir;
    }

    private File copy(Uri uri, File dir) throws Exception {
        ContentResolver cr = getContentResolver();
        String name = displayName(cr, uri);
        if (name == null || name.length() == 0) {
            name = "media";
        }
        // The importer goes by the extension.
        if (name.lastIndexOf('.') < 0) {
            String ext = MimeTypeMap.getSingleton().getExtensionFromMimeType(cr.getType(uri));
            if (ext != null) {
                name = name + "." + ext;
            }
        }
        name = name.replaceAll("[\\\\/:*?\"<>|]", "_");
        File dst = new File(dir, System.currentTimeMillis() + "_" + name);
        InputStream in = cr.openInputStream(uri);
        if (in == null) {
            return null;
        }
        try {
            OutputStream os = new FileOutputStream(dst);
            try {
                byte[] buf = new byte[1 << 16];
                int n;
                while ((n = in.read(buf)) > 0) {
                    os.write(buf, 0, n);
                }
            } finally {
                os.close();
            }
        } finally {
            in.close();
        }
        return dst;
    }

    private String displayName(ContentResolver cr, Uri uri) {
        Cursor c = null;
        try {
            c = cr.query(uri, new String[] {OpenableColumns.DISPLAY_NAME}, null, null, null);
            if (c != null && c.moveToFirst()) {
                return c.getString(0);
            }
        } catch (Exception e) {
            // No display name: the caller falls back to a generic one.
        } finally {
            if (c != null) {
                c.close();
            }
        }
        return null;
    }

    /** Written last, and renamed into place, so Rust never reads half a list. */
    private void writeResult(List<String> paths) {
        try {
            File dir = importsDir();
            File tmp = new File(dir, "picked.tmp");
            FileOutputStream os = new FileOutputStream(tmp);
            try {
                StringBuilder sb = new StringBuilder();
                for (String p : paths) {
                    sb.append(p).append('\n');
                }
                os.write(sb.toString().getBytes("UTF-8"));
            } finally {
                os.close();
            }
            File dst = new File(dir, "picked.txt");
            if (!tmp.renameTo(dst)) {
                Log.e(TAG, "could not publish the picker result");
            }
        } catch (Exception e) {
            Log.e(TAG, "could not write the picker result: " + e);
        }
    }
}
