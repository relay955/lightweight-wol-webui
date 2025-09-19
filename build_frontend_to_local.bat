set FRONTEND_DIR=frontend
set BUILD_DIR=frontend/static
set DEST_DIR=static

echo Frontend 디렉터리: %FRONTEND_DIR%
echo 빌드 디렉터리: %BUILD_DIR%
echo 대상 디렉터리: %DEST_DIR%

REM 1) 프론트엔드 빌드 (pnpm 사용 예시)
pushd %FRONTEND_DIR%
pnpm install
pnpm build
popd

if exist "%DEST_DIR%" rmdir /S /Q "%DEST_DIR%" & mkdir "%DEST_DIR%"
xcopy "%BUILD_DIR%\*" "%DEST_DIR%\" /E /I /Y

