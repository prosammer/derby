export class GPTContentProcessor {
  private static extractContent(jsonString: string): string {
    try {
      // Parse the input string as JSON
      const data = JSON.parse(jsonString);

      // Check if 'choices' and 'delta' fields exist and that 'content' is a string
      if (data.choices && data.choices[0] && data.choices[0].delta
        && typeof data.choices[0].delta.content === 'string') {
        return data.choices[0].delta.content;
      }

      return ''; // Return empty if content is not available
    } catch (error) {
      console.error('Failed to parse JSON or extract content', error);
      return '';
    }
  }

  public processChunk(chunk: string): string {
    // Remove any non-JSON prefix like "Chunk: data: " from the chunk before parsing
    const jsonChunk = chunk.replace(/^[^{}]+/, '');

    // Use the static method to extract content from the chunk
    return GPTContentProcessor.extractContent(jsonChunk);
  }
}