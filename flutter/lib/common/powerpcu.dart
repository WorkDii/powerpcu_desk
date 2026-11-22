import 'package:flutter/material.dart';
import 'package:flutter_hbb/consts.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:get/get.dart';
import 'package:url_launcher/url_launcher_string.dart';

const _kSourceUrl = 'https://github.com/WorkDii/powerpcu_desk';

bool isPowerPcuBuild() =>
    bind.mainGetBuildinOption(key: kOptionPowerPcu) == 'Y';

// Plain strings on purpose: translate() would swap "RustDesk" for the app name.
List<Widget> powerPcuAboutLines(TextStyle linkStyle) => [
      SelectionArea(
              child: Text('PowerPCU Remote — based on RustDesk · AGPL-3.0'))
          .marginSymmetric(vertical: 4.0),
      InkWell(
          onTap: () {
            launchUrlString(_kSourceUrl);
          },
          child: Text(
            'Source code: $_kSourceUrl',
            style: linkStyle,
          ).marginSymmetric(vertical: 4.0)),
    ];
