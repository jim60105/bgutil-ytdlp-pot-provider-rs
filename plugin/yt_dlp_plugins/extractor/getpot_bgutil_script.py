from __future__ import annotations

import functools
import json
import os.path
import shutil
import subprocess

from yt_dlp.extractor.youtube.pot.provider import (
    PoTokenProviderError,
    PoTokenRequest,
    PoTokenResponse,
    register_preference,
    register_provider,
)
from yt_dlp.extractor.youtube.pot.utils import get_webpo_content_binding
from yt_dlp.utils import Popen

from yt_dlp_plugins.extractor.getpot_bgutil import BgUtilPTPBase


@register_provider
class BgUtilScriptPTP(BgUtilPTPBase):
    PROVIDER_NAME = 'bgutil:script'

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._check_script = functools.cache(self._check_script_impl)

    @functools.cached_property
    def _script_path(self):
        script_path = self._configuration_arg(
            'script_path', casesense=True, default=[None])[0]

        if script_path:
            return os.path.expandvars(script_path)

        # check deprecated arg
        deprecated_script_path = self.ie._configuration_arg(
            ie_key='youtube', key='getpot_bgutil_script', default=[None])[0]

        if deprecated_script_path:
            self._warn_and_raise(
                "'youtube:getpot_bgutil_script' extractor arg is deprecated, "
                "use 'youtubepot-bgutilscript:script_path' instead")

        # default if no arg was passed
        # Check common locations for the Rust executable
        possible_paths = [
            'bgutil-pot-generate',  # if in PATH
            os.path.join(
                os.getcwd(), 'target', 'debug', 'bgutil-pot-generate'
            ),
            os.path.join(
                os.getcwd(), 'target', 'release', 'bgutil-pot-generate'
            ),
            os.path.expanduser(
                '~/bgutil-ytdlp-pot-provider/target/debug/bgutil-pot-generate'
            ),
            os.path.expanduser(
                '~/bgutil-ytdlp-pot-provider/target/release/'
                'bgutil-pot-generate'
            ),
        ]
        
        for path in possible_paths:
            if shutil.which(path) or os.path.isfile(path):
                self.logger.debug(f'Found bgutil-pot-generate at: {path}')
                return path
        
        # Fallback to first option if none found
        default_path = possible_paths[0]
        self.logger.debug(
            f'No script path passed, defaulting to {default_path}')
        return default_path

    def is_available(self):
        return self._check_script(self._script_path)

    @functools.cached_property
    def _executable_path(self):
        executable_path = shutil.which(self._script_path)
        if executable_path:
            return executable_path
        elif os.path.isfile(self._script_path):
            return self._script_path
        return None

    def _check_script_impl(self, script_path):
        executable_path = self._executable_path
        if not executable_path:
            self.logger.debug(
                f"Executable path doesn't exist: {script_path}")
            return False
        
        stdout, stderr, returncode = Popen.run(
            [executable_path, '--version'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=self._GET_SERVER_VSN_TIMEOUT
        )
        if returncode:
            self.logger.warning(
                f'Failed to check executable version. '
                f'Executable returned {returncode} exit status. '
                f'stdout: {stdout}; stderr: {stderr}',
                once=True)
            return False
        else:
            self.logger.debug(f'bgutil-pot-generate version: {stdout.strip()}')
            return True

    def _real_request_pot(
        self,
        request: PoTokenRequest,
    ) -> PoTokenResponse:
        # used for CI check
        self.logger.trace(
            f'Generating POT via Rust executable: {self._script_path}')

        executable_path = self._executable_path
        if not executable_path:
            raise PoTokenProviderError(
                f'Executable not found: {self._script_path}')

        command_args = [executable_path]
        if proxy := request.request_proxy:
            command_args.extend(['-p', proxy])
        command_args.extend(['-c', get_webpo_content_binding(request)[0]])
        if request.bypass_cache:
            command_args.append('--bypass-cache')
        if request.request_source_address:
            command_args.extend(
                ['--source-address', request.request_source_address])
        if request.request_verify_tls is False:
            command_args.append('--disable-tls-verification')

        self.logger.info(
            f'Generating a {request.context.value} PO Token for '
            f'{request.internal_client_name} client via bgutil '
            f'Rust executable',
        )
        self.logger.debug(
            f'Executing command to get POT via Rust executable: '
            f'{" ".join(command_args)}'
        )

        try:
            stdout, stderr, returncode = Popen.run(
                command_args,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=self._GETPOT_TIMEOUT
            )
        except subprocess.TimeoutExpired as e:
            raise PoTokenProviderError(
                f'_get_pot_via_script failed: Timeout expired when trying '
                f'to run executable (caused by {e!r})'
            )
        except Exception as e:
            raise PoTokenProviderError(
                f'_get_pot_via_script failed: Unable to run executable '
                f'(caused by {e!r})'
            ) from e

        msg = ''
        if stdout_extra := stdout.strip().splitlines()[:-1]:
            msg = f'stdout:\n{stdout_extra}\n'
        if stderr_stripped := stderr.strip():  # Empty strings are falsy
            msg += f'stderr:\n{stderr_stripped}\n'
        msg = msg.strip()
        if msg:
            self.logger.trace(msg)
        if returncode:
            raise PoTokenProviderError(
                f'_get_pot_via_script failed with returncode {returncode}')

        try:
            json_resp = stdout.splitlines()[-1]
            self.logger.trace(f'JSON response:\n{json_resp}')
            # The JSON response is always the last line
            script_data_resp = json.loads(json_resp)
        except json.JSONDecodeError as e:
            raise PoTokenProviderError(
                f'Error parsing JSON response from _get_pot_via_script '
                f'(caused by {e!r})'
            ) from e
        if 'poToken' not in script_data_resp:
            raise PoTokenProviderError(
                'The executable did not respond with a po_token')
        return PoTokenResponse(po_token=script_data_resp['poToken'])


@register_preference(BgUtilScriptPTP)
def bgutil_script_getpot_preference(provider, request):
    return 1


__all__ = [BgUtilScriptPTP.__name__,
           bgutil_script_getpot_preference.__name__]
