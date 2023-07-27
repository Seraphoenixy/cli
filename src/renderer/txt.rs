use std::fmt;

use color_eyre::eyre::Result;
use novel_api::Timing;
use tokio::fs;
use tracing::info;

use crate::{
    cmd::Convert,
    utils::{self, Content, Novel, UNIX_LINE_BREAK, WINDOWS_LINE_BREAK},
};

pub async fn generate_txt(novel: Novel, convert: &Vec<Convert>) -> Result<()> {
    let mut timing = Timing::new();

    let markdown_file_path = utils::to_txt_file_name(&novel.name);

    let mut buf = String::with_capacity(4 * 1024 * 1024);

    write_metadata(&novel, &mut buf).await?;
    write_introduction(&novel, &mut buf, convert).await?;
    write_chapters(&novel, &mut buf).await?;
    // last \n
    buf.pop();

    if cfg!(windows) {
        buf = buf.replace(UNIX_LINE_BREAK, WINDOWS_LINE_BREAK);
    }
    fs::write(markdown_file_path, &buf).await?;

    info!(
        "Time spent on `generate txt`: {}",
        timing.elapsed()?
    );

    Ok(())
}

async fn write_metadata<T>(novel: &Novel, mut buf: T) -> Result<()>
where
    T: fmt::Write,
{
    buf.write_str(novel.name.as_str())?;
    buf.write_str(&format!("\n作者：{}\n", novel.author_name.as_str()))?;
    buf.write_str("来源：未知\n")?;

    Ok(())
}

async fn write_introduction<T>(novel: &Novel, mut buf: T, convert: &Vec<Convert>) -> Result<()>
where
    T: fmt::Write,
{
    if let Some(ref introduction) = novel.introduction {
        buf.write_str(&format!("{}\n", utils::convert_str("简介：", convert)?))?;

        for line in introduction {
            buf.write_str(line)?;
            buf.write_str("\n")?;
        }
        buf.write_str("\n\n\n\n\n")?;
    }

    Ok(())
}

async fn write_chapters<T>(novel: &Novel, mut buf: T) -> Result<()>
where
    T: fmt::Write,
{
    for volume in &novel.volumes {
        if !volume.chapters.is_empty() {
            for chapter in &volume.chapters {
                buf.write_str(&format!("{} : {}\n\n", volume.title, chapter.title))?;

                for content in chapter.contents.read().await.iter() {
                    match content {
                        Content::Text(line) => {
                            buf.write_str("　　")?;
                            buf.write_str(line)?;
                            buf.write_str("\n")?;
                        }
                        Content::Image(image) => {
                            let _ = &image.file_name;
                            //buf.write_str(&super::image_markdown_str(&image.file_name))?;
                        }
                    }
                }
                buf.write_str("\n\n\n")?;
            }
        }
    }

    Ok(())
}
