with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
        name="supershell",
        version="0.1.0",
        author="J. Alex Long",
        author_email="jalexlong@proton.me",
        description="An interactive game to learn bash.",
        long_description=long_description,
        long_description_content_type="text/markdown",
        packages=find_packages(),
        install_requires=[
            "rich",
        ],
        classifiers=[
            "Programming Language :: Python :: 3",
            "Operating System :: Linux",
        ],
        python_requires=">=3.13",
)
