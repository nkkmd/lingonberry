from pathlib import Path

path = Path("packages/protocol/src/lib.rs")
text = path.read_text()
if not text.endswith("}\n"):
    raise SystemExit("protocol source did not end with the test module delimiter")

tests = r'''

    #[test]
    fn signature_workspace_cleans_up_and_is_owner_only() {
        let workspace = SignatureWorkspace::create().expect("workspace must be created");
        let path = workspace.path().to_path_buf();
        assert!(path.is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
        drop(workspace);
        assert!(!path.exists());
    }

    #[test]
    fn signature_artifacts_reject_existing_paths_and_cleanup() {
        let workspace = SignatureWorkspace::create().expect("workspace must be created");
        let root = workspace.path().to_path_buf();
        let artifact = root.join("artifact.bin");
        write_new_file(&artifact, b"first").expect("first create must succeed");
        assert!(write_new_file(&artifact, b"second").is_err());
        drop(workspace);
        assert!(!root.exists());
    }

    #[test]
    fn concurrent_signature_workspaces_are_isolated_and_cleaned() {
        let handles = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let workspace = SignatureWorkspace::create().expect("workspace must be created");
                    let path = workspace.path().to_path_buf();
                    assert!(path.is_dir());
                    drop(workspace);
                    assert!(!path.exists());
                    path
                })
            })
            .collect::<Vec<_>>();
        let paths = handles
            .into_iter()
            .map(|handle| handle.join().expect("workspace thread must complete"))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(paths.len(), 8);
    }
'''

text = text[:-2] + tests + "}\n"
path.write_text(text)
