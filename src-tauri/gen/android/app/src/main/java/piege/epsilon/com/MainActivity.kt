package piege.epsilon.com

import android.Manifest
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.Settings
import android.widget.Toast
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.enableEdgeToEdge
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
  private lateinit var permissionLauncher: ActivityResultLauncher<Array<String>>

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    permissionLauncher = registerForActivityResult(
      ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
      val granted = permissions.values.all { it }
      if (!granted) {
        Toast.makeText(
          this,
          "Permission de stockage nécessaire pour fonctionner.",
          Toast.LENGTH_LONG
        ).show()
      }
    }

    requestStoragePermissions()
  }

  private fun requestStoragePermissions() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      if (!Environment.isExternalStorageManager()) {
        try {
          val intent = Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION)
          intent.data = android.net.Uri.parse("package:$packageName")
          startActivity(intent)
        } catch (e: Exception) {
          val intent = Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION)
          startActivity(intent)
        }
      }
    } else {
      val neededPermissions = mutableListOf(Manifest.permission.READ_EXTERNAL_STORAGE)
      if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
        neededPermissions.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
      }

      val notGranted = neededPermissions.filter {
        ContextCompat.checkSelfPermission(this, it) != android.content.pm.PackageManager.PERMISSION_GRANTED
      }

      if (notGranted.isNotEmpty()) {
        permissionLauncher.launch(notGranted.toTypedArray())
      }
    }
  }
}
