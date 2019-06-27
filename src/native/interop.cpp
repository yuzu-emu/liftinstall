/**
 * Misc interop helpers.
**/

#include "windows.h"
#include "winnls.h"
#include "shobjidl.h"
#include "objbase.h"
#include "objidl.h"
#include "shlguid.h"
#include "shlobj.h"

// https://stackoverflow.com/questions/52101827/windows-10-getsyscolor-does-not-get-dark-ui-color-theme
extern "C" int isDarkThemeActive()
{
    DWORD type;
    DWORD value;
    DWORD count = 4;
    LSTATUS st = RegGetValue(
        HKEY_CURRENT_USER,
        TEXT("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
        TEXT("AppsUseLightTheme"),
        RRF_RT_REG_DWORD,
        &type,
        &value,
        &count);
    if (st == ERROR_SUCCESS && type == REG_DWORD)
        return value == 0;
    return false;
}

extern "C" int saveShortcut(
    const char *shortcutPath,
    const char *description,
    const char *path,
    const char *args,
    const char *workingDir)
{
    char *errStr = NULL;
    HRESULT h;
    IShellLink *shellLink = NULL;
    IPersistFile *persistFile = NULL;

#ifdef _WIN64
    wchar_t wName[MAX_PATH + 1];
#else
    WORD wName[MAX_PATH + 1];
#endif

    int id;

    // Initialize the COM library
    h = CoInitialize(NULL);
    if (FAILED(h))
    {
        errStr = "Failed to initialize COM library";
        goto err;
    }

    h = CoCreateInstance(CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER,
                         IID_IShellLink, (PVOID *)&shellLink);
    if (FAILED(h))
    {
        errStr = "Failed to create IShellLink";
        goto err;
    }

    h = shellLink->QueryInterface(IID_IPersistFile, (PVOID *)&persistFile);
    if (FAILED(h))
    {
        errStr = "Failed to get IPersistFile";
        goto err;
    }

    //Append the shortcut name to the folder
    MultiByteToWideChar(CP_UTF8, 0, shortcutPath, -1, wName, MAX_PATH);

    // Load the file if it exists, to get the values for anything
    // that we do not set.  Ignore errors, such as if it does not exist.
    h = persistFile->Load(wName, 0);

    // Set the fields for which the application has set a value
    if (description != NULL)
        shellLink->SetDescription(description);
    if (path != NULL)
        shellLink->SetPath(path);
    if (args != NULL)
        shellLink->SetArguments(args);
    if (workingDir != NULL)
        shellLink->SetWorkingDirectory(workingDir);

    //Save the shortcut to disk
    h = persistFile->Save(wName, TRUE);
    if (FAILED(h))
    {
        errStr = "Failed to save shortcut";
        goto err;
    }

    persistFile->Release();
    shellLink->Release();
    CoUninitialize();
    return h;

err:
    if (persistFile != NULL)
        persistFile->Release();
    if (shellLink != NULL)
        shellLink->Release();
    CoUninitialize();

    return h;
}

extern "C" int spawnDetached(const wchar_t *app, const wchar_t *cmdline)
{
    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    // make non-constant copy of the parameters
    // this is allowed per https://docs.microsoft.com/en-us/windows/desktop/api/processthreadsapi/nf-processthreadsapi-createprocessw#security-remarks
    wchar_t *app_copy = wcsdup(app);
    wchar_t *cmdline_copy = wcsdup(cmdline);

    if (app_copy == NULL || cmdline_copy == NULL)
    {
        return GetLastError();
    }

    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    ZeroMemory(&pi, sizeof(pi));

    if (!CreateProcessW(app,              // module name
                        (LPWSTR)cmdline,  // Command line, unicode is allowed
                        NULL,             // Process handle not inheritable
                        NULL,             // Thread handle not inheritable
                        FALSE,            // Set handle inheritance to FALSE
                        CREATE_NO_WINDOW, // Create without window
                        NULL,             // Use parent's environment block
                        NULL,             // Use parent's starting directory
                        &si,              // Pointer to STARTUPINFO structure
                        &pi)              // Pointer to PROCESS_INFORMATION structure
    )
    {
        return GetLastError();
    }

    // Close process and thread handles.
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    return 0;
}

extern "C" HRESULT getSystemFolder(wchar_t *out_path)
{
    PWSTR path = NULL;
    HRESULT result = SHGetKnownFolderPath(FOLDERID_System, 0, NULL, &path);
    if (result == S_OK)
    {
        wcscpy_s(out_path, MAX_PATH + 1, path);
        CoTaskMemFree(path);
    }
    return result;
}
